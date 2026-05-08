use std::{env};

use crate::utils;

pub mod echo;

pub enum Command<'a> {
    Exit,
    Echo(Vec<&'a str>),
    Type(Vec<&'a str>),
    Unkown,
}

impl<'a> Command<'a> {
    pub fn from_raw(input: &'a str) -> Command<'a> {
        let args: Vec<&'a str> = input.trim().split_whitespace().collect();

        match args[0] {
            "exit" => Command::Exit,

            "echo" => Command::Echo(args[1..].to_vec()),

            "type" => Command::Type(args[1..].to_vec()),

            _ => Command::Unkown,
        }
    }
}

pub fn handle_type(args: Vec<&str>) -> String {
    let path = env::var("PATH").unwrap();
    match Command::from_raw(args[0]) {
        Command::Unkown => {
            for dir in env::split_paths(&path) {
                let path = dir.join(args[0]);
                if utils::is_executable(&path) {
                    return format!("{} is {}", args[0], path.display());
                }
            }
            format!("{}: not found", args[0])
        }
        _ => format!("{} is a shell builtin", args[0]),
    }
}
