use std::{env, process};

use crate::utils::{self, parse_args};

pub mod cd;
pub mod echo;
pub mod pwd;

pub enum Command {
    Exit,
    Echo(Vec<String>),
    Type(Vec<String>),
    Pwd(Vec<String>),
    Cd(Vec<String>),
    Unknown(String, Vec<String>),
}

impl<'a> Command {
    pub fn from_raw(input: &String) -> Command {
        let args = parse_args(input.trim());

        match args[0].as_str() {
            "exit" => Command::Exit,

            "echo" => Command::Echo(args[1..].to_vec()),

            "type" => Command::Type(args[1..].to_vec()),

            "pwd" => Command::Pwd(args[1..].to_vec()),

            "cd" => Command::Cd(args[1..].to_vec()),

            _ => Command::Unknown(args[0].clone(), args[1..].to_vec()),
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
        Command::Unknown(_, _) => {
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

pub fn handle_run(cmd: String, args: Vec<String>) {
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
            println!("{}: command not found", cmd);
        }
    }
}
