use std::collections::BTreeSet;
use std::{ process};

use crate::cmd::context::Context;
use crate::utils;

pub mod cd;
pub mod context;
pub mod echo;
pub mod pwd;

pub enum BuiltInCommand {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
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

            _ => Self::Unknown,
        }
    }
}

impl BuiltInCommand {
    pub fn matches(prefix: &str) -> BTreeSet<String> {
        let cmds = ["exit", "echo", "type", "pwd", "cd"];
        cmds.iter()
            .filter(|c| c.starts_with(&prefix))
            .map(ToString::to_string)
            .collect()
    }
}
pub fn handle_type(ctx: &mut Context) -> Result<(), String> {
    if ctx.args.is_empty() {
        return Ok(());
    }
    // let path = env::var("PATH").unwrap();
    let arg = &ctx.args[0];
    match BuiltInCommand::from(arg.as_str()) {
        BuiltInCommand::Unknown => {
            if let Some(base_path) = utils::find_possible_path_to_command(arg) {
                return ctx
                    .stdout
                    .writeln(&format!("{} is {}", arg, base_path))
                    .map_err(|e| e.to_string());
            }
            return Err(format!("{}: not found", arg));
        }
        _ => ctx
            .stdout
            .writeln(&format!("{} is a shell builtin", arg))
            .map_err(|e| e.to_string()),
    }
}

pub fn handle_run(cmd: String, ctx: &mut Context) -> Result<(), String> {
    let output = process::Command::new(&cmd).args(&ctx.args).output();

    match output {
        Ok(output) => {
            ctx.stdout
                .write(&String::from_utf8_lossy(&output.stdout))
                .map_err(|e| e.to_string())?;
            ctx.stderr
                .write(&String::from_utf8_lossy(&output.stderr))
                .map_err(|e| e.to_string())?;
            Ok(())
        }

        Err(_) => Err(format!("{}: command not found", cmd)),
    }
}

pub fn dispatch(cmd_name: &str, ctx: &mut Context) -> Result<(), String> {
    let command = BuiltInCommand::from(cmd_name);
    match command {
        BuiltInCommand::Exit => {
            std::process::exit(0);
        }
        BuiltInCommand::Echo => echo::handle_echo(ctx),
        BuiltInCommand::Type => handle_type(ctx),
        BuiltInCommand::Pwd => pwd::handle_pwd(ctx),
        BuiltInCommand::Cd => cd::handle_cd(ctx),
        BuiltInCommand::Unknown => handle_run(cmd_name.to_string(), ctx),
    }
}
