use std::{env, process};

use crate::utils;

pub mod echo;
pub mod pwd;

pub enum Command<'a> {
    Exit,
    Echo(Vec<&'a str>),
    Type(Vec<&'a str>),
    Pwd(Vec<&'a str>),
    Unknown(&'a str, Vec<&'a str>),
}

impl<'a> Command<'a> {
    pub fn from_raw(input: &'a str) -> Command<'a> {
        let args: Vec<&'a str> = input.trim().split_whitespace().collect();

        match args[0] {
            "exit" => Command::Exit,

            "echo" => Command::Echo(args[1..].to_vec()),

            "type" => Command::Type(args[1..].to_vec()),

            "pwd" => Command::Pwd(args[1..].to_vec()),

            _ => Command::Unknown(args[0], args[1..].to_vec()),
        }
    }
}

pub enum Type<'a> {
    Exe(&'a str, String),
    BuiltIn(&'a str),
    Unknown(&'a str),
}
pub fn handle_type(args: Vec<&'_ str>) -> Type<'_> {
    let path = env::var("PATH").unwrap();
    match Command::from_raw(args[0]) {
        Command::Unknown(_, _) => {
            for dir in env::split_paths(&path) {
                let path = dir.join(args[0]);
                if utils::is_executable(&path) {
                    // return format!("{} is {}", args[0], path.display());
                    return Type::Exe(args[0], path.display().to_string());
                }
            }
            // format!("{}: not found", args[0])
            return Type::Unknown(args[0]);
        }
        // _ => format!("{} is a shell builtin", args[0]),
        _ => Type::BuiltIn(args[0]),
    }
}

pub fn handle_run(cmd: &str, args: Vec<&str>) {
    let program = process::Command::new(cmd)
        .args(args)
        .stdin(process::Stdio::inherit())
        .stdout(process::Stdio::inherit())
        .spawn();

    match program {
        Ok(mut program) => {
            let _ = program.wait();
        }
        Err(_) => {
            println!("{cmd}: command not found");
        }
    }
}
