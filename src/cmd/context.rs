use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

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

pub struct Context<'a> {
    pub args: Vec<String>,
    pub stdout: OutputDestination,
    pub stderr: OutputDestination,
    pub completions: &'a mut HashMap<String, PathBuf>,
}

impl<'a> Context<'a> {
    pub fn new(args: Vec<String>, completions: &'a mut HashMap<String, PathBuf>) -> Self {
        Self {
            args,
            stdout: OutputDestination::Stdout,
            stderr: OutputDestination::Stderr,
            completions,
        }
    }
}
