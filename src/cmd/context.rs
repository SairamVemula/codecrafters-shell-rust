use std::collections::{BTreeSet, HashMap, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::{self, PipeReader, PipeWriter, Write};
use std::path::PathBuf;
use std::process::Child;
use std::sync::{Arc, Mutex};

use anyhow::{Result, anyhow};
use console::Term;

use crate::cmd::jobs::Job;
use crate::utils::{self, unparse_args};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RedirectionType {
    StdoutWrite,
    StdoutAppend,
    StderrWrite,
    StderrAppend,
    StdoutPipe,
    StdinPipe,
}

#[derive(Debug)]
pub struct Redirection {
    pub r_type: RedirectionType,
    pub file: Option<String>,
    pub pipe_reader: Option<PipeReader>,
    pub pipe_writer: Option<PipeWriter>,
}

impl Clone for Redirection {
    fn clone(&self) -> Self {
        Self {
            r_type: self.r_type,
            file: self.file.clone(),
            pipe_reader: None,
            pipe_writer: None,
        }
    }
}

#[derive(Debug)]
pub enum OutputDestination {
    Stdout,
    Stderr,
    File(File),
    Piped(Option<PipeWriter>),
}

impl OutputDestination {
    pub fn write(&mut self, content: &str) -> io::Result<()> {
        match self {
            OutputDestination::Stdout => {
                let mut stdout = io::stdout();
                write!(stdout, "{}", content)?;
                stdout.flush()
            }
            OutputDestination::Stderr => {
                let mut stderr = io::stderr();
                write!(stderr, "{}", content)?;
                stderr.flush()
            }
            OutputDestination::File(file) => {
                write!(file, "{}", content)?;
                file.flush()
            }
            OutputDestination::Piped(writer) => {
                if let Some(writer) = writer {
                    write!(writer, "{}", content)?;
                    return writer.flush();
                }
                Ok(())
            }
        }
    }

    pub fn writeln(&mut self, content: &str) -> io::Result<()> {
        self.write(content)?;
        self.write("\n")
    }
}

pub type CompletionStore = HashMap<String, PathBuf>;

pub struct AppState {
    pub completions: CompletionStore,
    pub jobs: VecDeque<Job>,
    pub next_job_id: BTreeSet<usize>,
}

pub type SharedState = Arc<Mutex<AppState>>;

impl AppState {
    pub fn new() -> SharedState {
        Arc::new(Mutex::new(Self {
            completions: CompletionStore::new(),
            jobs: VecDeque::new(),
            next_job_id: BTreeSet::new(),
        }))
    }
}

pub struct Context {
    pub name: String,
    pub args: Vec<String>,
    pub stdin: Option<PipeReader>,
    pub stdout: OutputDestination,
    pub stderr: OutputDestination,
    pub state: SharedState,
    pub is_job: bool,
    pub pipes: Vec<Context>,
    pub redirections: Vec<Redirection>,
}

impl Context {
    pub fn new(state: SharedState, args: Option<Vec<String>>) -> Result<Self> {
        let (mut parsed, recursive) = match args {
            Some(args) => (args, true),
            None => {
                let input = {
                    let mut term = Term::stdout();
                    let mut guard = state.lock().map_err(|_| anyhow!("Lock poisoned"))?;
                    utils::get_user_input(&mut term, &mut guard.completions)?
                };
                (utils::parse_args(input.trim()), false)
            }
        };

        if parsed.is_empty() {
            return Err(anyhow!("empty input"));
        }
        let name = parsed.remove(0);

        let (pipes, mut redirections) = utils::parse_redirections(&mut parsed);

        let is_job = if let Some(s) = parsed.last()
            && s == "&"
        {
            parsed.remove(parsed.len() - 1);
            true
        } else {
            false
        };

        // ctx
        let mut ctx = Self {
            name,
            args: parsed,
            stdout: OutputDestination::Stdout,
            stderr: OutputDestination::Stderr,
            state: state.clone(),
            is_job,
            pipes: vec![],
            redirections: vec![],
            stdin: None,
        };

        if !recursive {
            if pipes.len() > 0 {
                let redirection = redirections.remove(0);
                ctx.apply_redirections(vec![redirection])?;

                for pipe in pipes {
                    let mut pipe_ctx = Self::new(state.clone(), Some(pipe.to_vec()))?;
                    let redirection = redirections.remove(0);
                    pipe_ctx.apply_redirections(vec![redirection])?;
                    ctx.pipes.push(pipe_ctx);
                }
            } else {
                ctx.apply_redirections(redirections.clone())?;
                ctx.redirections = redirections;
            }
        }
        Ok(ctx)
    }

    pub fn apply_redirections(&mut self, redirections: Vec<Redirection>) -> io::Result<()> {
        for redir in redirections {
            let append = matches!(
                redir.r_type,
                RedirectionType::StdoutAppend | RedirectionType::StderrAppend
            );

            match redir.r_type {
                RedirectionType::StdoutWrite | RedirectionType::StdoutAppend => {
                    if let Some(path) = redir.file {
                        let file = OpenOptions::new()
                            .create(true)
                            .write(true)
                            .append(append)
                            .truncate(!append)
                            .open(path)?;
                        self.stdout = OutputDestination::File(file);
                    }
                }
                RedirectionType::StderrWrite | RedirectionType::StderrAppend => {
                    if let Some(path) = redir.file {
                        let file = OpenOptions::new()
                            .create(true)
                            .write(true)
                            .append(append)
                            .truncate(!append)
                            .open(path)?;
                        self.stderr = OutputDestination::File(file);
                    }
                }
                RedirectionType::StdoutPipe => {
                    self.stdout = OutputDestination::Piped(redir.pipe_writer);
                    self.stdin = redir.pipe_reader;
                }
                RedirectionType::StdinPipe => {
                    self.stdin = redir.pipe_reader;
                }
            }
        }
        Ok(())
    }

    pub fn to_string(&self) -> String {
        let mut parts = vec![self.name.clone()];
        parts.extend(self.args.clone());
        let mut cmd = unparse_args(&parts);

        for pipe in &self.pipes {
            cmd.push_str(&format!(" | {}", pipe.to_string()));
        }

        for redir in &self.redirections {
            if let Some(file) = &redir.file {
                let symbol = match redir.r_type {
                    RedirectionType::StdoutWrite => ">",
                    RedirectionType::StdoutAppend => ">>",
                    RedirectionType::StderrWrite => "2>",
                    RedirectionType::StderrAppend => "2>>",
                    _ => continue,
                };
                cmd.push_str(&format!(" {} {}", symbol, file));
            }
        }

        if self.is_job {
            cmd.push_str(" &");
        }

        cmd
    }

    pub fn add_job(&mut self, process: Child) -> (usize, u32) {
        let mut state = self.state.lock().unwrap();
        let pid = process.id();
        let id = state
            .next_job_id
            .pop_first()
            .unwrap_or(state.jobs.len() + 1);
        let job = Job::new(id, self.to_string(), process);
        state.jobs.push_back(job);
        return (id, pid);
    }
}
