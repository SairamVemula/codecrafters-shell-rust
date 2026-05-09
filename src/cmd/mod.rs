use std::{
    env,
    process,
};

use crate::utils;
use crate::cmd::context::Context;

pub mod cd;
pub mod echo;
pub mod pwd;
pub mod context;

pub enum Command {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
    Unknown,
}

impl<'a> Command {
    pub fn from_raw(input: &str) -> Command {
        match input {
            "exit" => Command::Exit,

            "echo" => Command::Echo,

            "type" => Command::Type,

            "pwd" => Command::Pwd,

            "cd" => Command::Cd,

            _ => Command::Unknown,
        }
    }
}

pub fn handle_type(ctx: &mut Context) -> Result<(), String> {
    if ctx.args.is_empty() {
        return Ok(());
    }
    let path = env::var("PATH").unwrap();
    let arg = &ctx.args[0];
    match Command::from_raw(arg) {
        Command::Unknown => {
            for dir in env::split_paths(&path) {
                let base_path = dir.join(arg);
                if base_path.exists() && utils::is_executable(&base_path) {
                    return ctx.stdout.writeln(&format!("{} is {}", arg, base_path.display())).map_err(|e| e.to_string());
                }
            }
            return Err(format!("{}: not found", arg));
        }
        _ => {
            ctx.stdout.writeln(&format!("{} is a shell builtin", arg)).map_err(|e| e.to_string())
        }
    }
}

pub fn handle_run(cmd: String, ctx: &mut Context) -> Result<(), String> {
    let output = process::Command::new(&cmd).args(&ctx.args).output();

    match output {
        Ok(output) => {
            ctx.stdout.write(&String::from_utf8_lossy(&output.stdout)).map_err(|e| e.to_string())?;
            ctx.stderr.write(&String::from_utf8_lossy(&output.stderr)).map_err(|e| e.to_string())?;
            Ok(())
        }

        Err(_) => Err(format!("{}: command not found", cmd)),
    }
}

pub fn dispatch(cmd_name: &str, ctx: &mut Context) -> Result<(), String> {
    let command = Command::from_raw(cmd_name);
    match command {
        Command::Exit => {
            std::process::exit(0);
        }
        Command::Echo => echo::handle_echo(ctx),
        Command::Type => handle_type(ctx),
        Command::Pwd => pwd::handle_pwd(ctx),
        Command::Cd => handle_cd_wrapper(ctx),
        Command::Unknown => handle_run(cmd_name.to_string(), ctx),
    }
}

fn handle_cd_wrapper(ctx: &mut Context) -> Result<(), String> {
    cd::handle_cd(ctx)
}
