use std::collections::{HashMap, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Child;

use anyhow::{Result, anyhow};
use console::Term;

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

pub struct Store {
    pub completions: CompletionStore,
    pub jobs: VecDeque<Child>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            completions: CompletionStore::new(),
            jobs: VecDeque::new(),
        }
    }
}

pub struct Context<'a> {
    pub original_input: String,
    pub name: String,
    pub args: Vec<String>,
    pub stdout: OutputDestination,
    pub stderr: OutputDestination,
    pub store: &'a mut Store,
    pub is_job: bool,
}

impl<'a> Context<'a> {
    pub fn new(store: &'a mut Store) -> Result<Self> {
        let mut term = Term::stdout();
        let input = utils::get_user_input(&mut term, &mut store.completions).unwrap();

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
            store,
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

    pub fn add_job(&mut self, job: Child) -> (usize, u32) {
        let pid = job.id();
        self.store.jobs.push_back(job);
        (self.store.jobs.len(), pid)
    }
}
