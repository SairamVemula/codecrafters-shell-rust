use std::collections::BTreeSet;
use std::process;
use anyhow::{anyhow, Result};

use crate::cmd::complete::Complete;
use crate::cmd::context::Context;
use crate::cmd::jobs::Jobs;
use crate::utils;

pub mod cd;
pub mod context;
pub mod echo;
pub mod pwd;
pub mod complete;
pub mod jobs;

pub enum BuiltInCommand {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
    Complete,
    Jobs,
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
            _ => Self::Unknown,
        }
    }
}

impl BuiltInCommand {
    const ALL: &'static [&'static str] = &["exit", "echo", "type", "pwd", "cd", "complete", "jobs"];

    pub fn matches(prefix: &str) -> BTreeSet<String> {
        Self::ALL.iter()
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

pub fn handle_run(cmd: String, ctx: &mut Context) -> Result<()> {
    let output = process::Command::new(&cmd).args(&ctx.args).output();

    match output {
        Ok(output) => {
            ctx.stdout.write(&String::from_utf8_lossy(&output.stdout))?;
            ctx.stderr.write(&String::from_utf8_lossy(&output.stderr))?;
            Ok(())
        }
        Err(_) => Err(anyhow!("{}: command not found", cmd)),
    }
}

pub fn dispatch(cmd_name: &str, ctx: &mut Context) -> Result<()> {
    match BuiltInCommand::from(cmd_name) {
        BuiltInCommand::Exit => {
            process::exit(0);
        }
        BuiltInCommand::Echo => echo::handle_echo(ctx),
        BuiltInCommand::Type => handle_type(ctx),
        BuiltInCommand::Pwd => pwd::handle_pwd(ctx),
        BuiltInCommand::Cd => cd::handle_cd(ctx),
        BuiltInCommand::Complete => Complete::handle(ctx),
        BuiltInCommand::Jobs => Jobs::handle(ctx),
        BuiltInCommand::Unknown => handle_run(cmd_name.to_string(), ctx),
    }
}
