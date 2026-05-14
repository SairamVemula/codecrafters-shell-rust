use std::{
    env,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
};

use anyhow::{Ok, Result};

use crate::cmd::context::{Context, SharedState};

pub enum HistoryFlag {
    Read,
    Write,
    Append,
    Print,
}
pub struct History {
    pub flag: HistoryFlag,
    pub file: Option<File>,
    pub limit: Option<usize>,
}

impl History {
    pub fn new() -> Option<Self> {
        let path = env::var("HISTFILE").ok()?;
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .append(true)
            .open(path)
            .ok()?; // Returns None if the file cannot be opened

        Some(Self {
            flag: HistoryFlag::Append,
            file: Some(file),
            limit: None,
        })
    }

    pub fn parse(args: &Vec<String>) -> Self {
        match args.get(0) {
            Some(first) => match first.as_str() {
                "-r" => Self {
                    flag: HistoryFlag::Read,
                    file: OpenOptions::new()
                        .read(true)
                        .open(args.get(1).unwrap())
                        .ok(),
                    limit: None,
                },
                "-w" => Self {
                    flag: HistoryFlag::Write,
                    file: OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(args.get(1).unwrap())
                        .ok(),
                    limit: None,
                },
                "-a" => Self {
                    flag: HistoryFlag::Append,
                    file: OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(true)
                        .open(args.get(1).unwrap())
                        .ok(),
                    limit: None,
                },
                _ => {
                    let limit = args.get(0).and_then(|s| s.parse::<usize>().ok());

                    Self {
                        flag: HistoryFlag::Print,
                        file: None,
                        limit,
                    }
                }
            },
            None => Self {
                flag: HistoryFlag::Print,
                file: None,
                limit: None,
            },
        }
    }

    pub fn handle(ctx: &mut Context) -> Result<()> {
        let mut h = History::parse(&ctx.args);
        match h.flag {
            HistoryFlag::Read => h.read(&mut ctx.state),
            HistoryFlag::Write => h.write(&mut ctx.state),
            HistoryFlag::Append => h.append(&mut ctx.state),
            HistoryFlag::Print => h.print(ctx),
        }
    }

    pub fn write(&mut self, state: &mut SharedState) -> Result<()> {
        if let Some(file) = &mut self.file {
            let guard = state.lock().unwrap();
            for line in &guard.history {
                writeln!(file, "{}", line)?;
            }
        }
        Ok(())
    }
    pub fn append(&mut self, state: &mut SharedState) -> Result<()> {
        if let Some(file) = &mut self.file {
            let mut guard = state.lock().unwrap();
            let pointer = guard.history_pointer;
            for line in &guard.history[pointer..] {
                writeln!(file, "{}", line)?;
            }
            guard.history_pointer = guard.history.len();
        }
        Ok(())
    }
    pub fn read(&mut self, state: &mut SharedState) -> Result<()> {
        if let Some(file) = &self.file {
            let reader = BufReader::new(file);

            let history = reader.lines().map(|l| l.ok()).flatten();

            let mut guard = state.lock().unwrap();
            guard.history.extend(history);
            guard.history_pointer = guard.history.len();
        }
        Ok(())
    }
    pub fn print(&mut self, ctx: &mut Context) -> Result<()> {
        let guard = ctx.state.lock().unwrap();
        let end = guard.history.len();

        let start = if let Some(n) = self.limit {
            end.saturating_sub(n)
        } else {
            0
        };

        for (i, cmd) in guard.history[start..end].iter().enumerate() {
            ctx.stdout
                .writeln(&format!("    {} {}", start + i + 1, cmd))?;
        }
        Ok(())
    }
}
