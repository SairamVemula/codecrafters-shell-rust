use std::{env, process};

use crate::utils;

pub mod cd;
pub mod echo;
pub mod pwd;

pub enum Command {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
    Unknown,
}

impl<'a> Command {
    pub fn from_raw(input: &String) -> Command {

        match input.as_str() {
            "exit" => Command::Exit,

            "echo" => Command::Echo,

            "type" => Command::Type,

            "pwd" => Command::Pwd,

            "cd" => Command::Cd,

            _ => Command::Unknown,
        }
    }
}

pub enum Type {
    Exe(String, String),
    BuiltIn(String),
    Unknown(String),
}
pub fn handle_type(args: Vec<String>) -> Type {
    let path = env::var("PATH").unwrap();
    match Command::from_raw(&args[0]) {
        Command::Unknown => {
            for dir in env::split_paths(&path) {
                let path = dir.join(args[0].clone());
                if utils::is_executable(&path) {
                    // return format!("{} is {}", args[0], path.display());
                    return Type::Exe(args[0].clone(), path.display().to_string());
                }
            }
            // format!("{}: not found", args[0])
            return Type::Unknown(args[0].clone());
        }
        // _ => format!("{} is a shell builtin", args[0]),
        _ => Type::BuiltIn(args[0].clone()),
    }
}

pub fn handle_run(cmd: String, args: Vec<String>) -> Result<String, String> {
    let program = process::Command::new(&cmd)
        .args(args)
        .stdin(process::Stdio::inherit())
        .stdout(process::Stdio::inherit())
        .spawn();

    match program {
        Ok(mut program) => {
            let _ = program.wait();
        }
        Err(_) => {
            return Err(format!("{}: command not found", cmd));
        }
    }

    Ok("".to_string())
}
