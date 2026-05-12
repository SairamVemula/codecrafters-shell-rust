use std::collections::{HashMap, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Child;
use std::sync::{Arc, Mutex};

use anyhow::{Result, anyhow};
use console::Term;

use crate::cmd::jobs::Job;
use crate::utils;

#[derive(Debug, PartialEq)]
pub enum RedirectionType {
    StdoutWrite,
    StdoutAppend,
    StderrWrite,
    StderrAppend,
}

#[derive(Debug)]
pub struct Redirection {
    pub r_type: RedirectionType,
    pub file: String,
}

pub enum OutputDestination {
    Stdout,
    Stderr,
    File(File),
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
    pub next_job_id: usize,
}

type SharedState = Arc<Mutex<AppState>>;

impl AppState {
    pub fn new() -> SharedState {
        Arc::new(Mutex::new(Self {
            completions: CompletionStore::new(),
            jobs: VecDeque::new(),
            next_job_id: 1,
        }))
    }
}

pub struct Context {
    pub original_input: String,
    pub name: String,
    pub args: Vec<String>,
    pub stdout: OutputDestination,
    pub stderr: OutputDestination,
    pub state: SharedState,
    pub is_job: bool,
}

impl Context {
    pub fn new(state: SharedState) -> Result<Self> {
        let mut term = Term::stdout();
        let input = {
            let mut state = state.lock().unwrap();
            utils::get_user_input(&mut term, &mut state.completions).unwrap()
        };

        let mut parsed = utils::parse_args(input.trim());

        if parsed.is_empty() {
            return Err(anyhow!("empty input"));
        }

        let name = parsed.remove(0);

        let redirections = utils::parse_redirections(&mut parsed);

        let is_job = if let Some(s) = parsed.last()
            && s == "&"
        {
            parsed.remove(parsed.len() - 1);
            true
        } else {
            false
        };

        let mut ctx = Self {
            original_input: input,
            name,
            args: parsed,
            stdout: OutputDestination::Stdout,
            stderr: OutputDestination::Stderr,
            state,
            is_job,
        };

        ctx.apply_redirections(redirections)?;
        Ok(ctx)
    }

    pub fn apply_redirections(&mut self, redirections: Vec<Redirection>) -> io::Result<()> {
        for redir in redirections {
            let append = matches!(
                redir.r_type,
                RedirectionType::StdoutAppend | RedirectionType::StderrAppend
            );
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(append)
                .truncate(!append)
                .open(&redir.file)?;

            match redir.r_type {
                RedirectionType::StdoutWrite | RedirectionType::StdoutAppend => {
                    self.stdout = OutputDestination::File(file);
                }
                RedirectionType::StderrWrite | RedirectionType::StderrAppend => {
                    self.stderr = OutputDestination::File(file);
                }
            }
        }
        Ok(())
    }

    pub fn add_job(&mut self, process: Child) -> (usize, u32) {
        let mut state = self.state.lock().unwrap();
        let id = state.next_job_id;
        let pid = process.id();
        state.next_job_id += 1;
        let job = Job::new(id, self.original_input.clone(), process);
        state.jobs.push_back(job);
        return (id, pid);
    }
}
