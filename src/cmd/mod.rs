use anyhow::{Ok, Result, anyhow};
use std::collections::BTreeSet;
use std::process::{self, Stdio};

use crate::cmd::complete::Complete;
use crate::cmd::context::{Context, OutputDestination};
use crate::cmd::history::History;
use crate::cmd::jobs::Job;
use crate::utils;

pub mod cd;
pub mod complete;
pub mod context;
pub mod echo;
pub mod history;
pub mod jobs;
pub mod pwd;

pub enum BuiltInCommand {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
    Complete,
    Jobs,
    History,
    Unknown,
}

impl From<&str> for BuiltInCommand {
    fn from(input: &str) -> Self {
        match input {
            "exit" => Self::Exit,
            "echo" => Self::Echo,
            "type" => Self::Type,
            "pwd" => Self::Pwd,
            "cd" => Self::Cd,
            "complete" => Self::Complete,
            "jobs" => Self::Jobs,
            "history" => Self::History,
            _ => Self::Unknown,
        }
    }
}

impl BuiltInCommand {
    const ALL: &'static [&'static str] = &[
        "exit", "echo", "type", "pwd", "cd", "complete", "jobs", "history",
    ];

    pub fn matches(prefix: &str) -> BTreeSet<String> {
        Self::ALL
            .iter()
            .filter(|c| c.starts_with(prefix))
            .map(|s| s.to_string())
            .collect()
    }
}

pub fn handle_type(ctx: &mut Context) -> Result<()> {
    if ctx.args.is_empty() {
        return Ok(());
    }
    let arg = &ctx.args[0];
    match BuiltInCommand::from(arg.as_str()) {
        BuiltInCommand::Unknown => {
            if let Some(base_path) = utils::find_possible_path_to_command(arg) {
                ctx.stdout.writeln(&format!("{} is {}", arg, base_path))?;
                return Ok(());
            }
            Err(anyhow!("{}: not found", arg))
        }
        _ => {
            ctx.stdout.writeln(&format!("{} is a shell builtin", arg))?;
            Ok(())
        }
    }
}

pub fn handle_run(ctx: &mut Context) -> Result<()> {
    let mut cmd = process::Command::new(&ctx.name);
    cmd.args(&ctx.args);

    match ctx.stdin.take() {
        Some(reader) => cmd.stdin(reader),
        None => cmd.stdin(Stdio::inherit()),
    };

    match &mut ctx.stdout {
        OutputDestination::Stdout => {
            cmd.stdout(Stdio::inherit());
        }
        OutputDestination::File(f) => {
            cmd.stdout(Stdio::from(f.try_clone()?));
        }
        OutputDestination::Piped(w) => {
            if let Some(w) = w.take() {
                cmd.stdout(w);
            }
        }
        _ => {}
    }

    match &ctx.stderr {
        OutputDestination::Stderr => {
            cmd.stderr(Stdio::inherit());
        }
        OutputDestination::File(f) => {
            cmd.stderr(Stdio::from(f.try_clone()?));
        }
        _ => {}
    }

    let child = cmd
        .spawn()
        .map_err(|_| anyhow!("{}: command not found", &ctx.name))?;

    if ctx.is_job {
        let (id, pid) = ctx.add_job(child);
        println!("[{}] {}", id, pid);
        return Ok(());
    }

    let mut child = child;
    child.wait()?;

    Ok(())
}

pub fn dispatch(ctx: &mut Context) -> Result<()> {
    match BuiltInCommand::from(ctx.name.as_str()) {
        BuiltInCommand::Exit => {
            if let Some(mut history) = History::new() {
                let _ = history.append(&mut ctx.state);
            }
            process::exit(0);
        }
        BuiltInCommand::Echo => echo::handle_echo(ctx),
        BuiltInCommand::Type => handle_type(ctx),
        BuiltInCommand::Pwd => pwd::handle_pwd(ctx),
        BuiltInCommand::Cd => cd::handle_cd(ctx),
        BuiltInCommand::Complete => Complete::handle(ctx),
        BuiltInCommand::Jobs => Job::handle(ctx),
        BuiltInCommand::History => History::handle(ctx),
        BuiltInCommand::Unknown => handle_run(ctx),
    }
}
