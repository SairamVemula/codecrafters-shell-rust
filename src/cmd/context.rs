use std::fs::File;
use std::io::{self, Write};

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

pub struct Context {
    pub args: Vec<String>,
    pub stdout: OutputDestination,
    pub stderr: OutputDestination,
}

impl Context {
    pub fn new(args: Vec<String>) -> Self {
        Self {
            args,
            stdout: OutputDestination::Stdout,
            stderr: OutputDestination::Stderr,
        }
    }
}
