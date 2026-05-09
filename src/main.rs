use std::{
    fs::File,
    io::{self, Write},
};

use crate::cmd::Command;

mod cmd;
mod utils;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let parsed = utils::parse_args(input.trim());

        if parsed.is_empty() {
            continue;
        }

        let cmd = parsed[0].clone();
        let command = Command::from_raw(&cmd);

        let mut args = parsed[1..].to_vec();

        let output_file = if let Some(pos) = args.iter().position(|t| t == ">" || t == "1>") {
            if pos + 1 >= args.len() {
                eprintln!("No output file specified");
                continue;
            }

            let filename = args[pos + 1].clone();

            args.drain(pos..=pos + 1);

            Some(File::create(filename).expect("Failed to create file"))
        } else {
            None
        };

        let result = match command {
            Command::Exit => {
                break;
            }
            Command::Echo => cmd::echo::handle_echo(args),
            Command::Type => match cmd::handle_type(args) {
                cmd::Type::Exe(cmd, path) => Ok(format!("{} is {}", cmd, path)),
                cmd::Type::Unknown(cmd) => Err(format!("{}: not found", cmd)),
                cmd::Type::BuiltIn(cmd) => Ok(format!("{} is a shell builtin", cmd)),
            },
            Command::Pwd => cmd::pwd::handle_pwd(args),
            Command::Cd => cmd::cd::handle_cd(args),
            Command::Unknown => cmd::handle_run(cmd.clone(), args),
        };

        match output_file {
            Some(mut file) => match result {
                Ok(s) => {
                    write!(file, "{}", s).ok();
                }
                Err(s) => {
                    write!(io::stderr(), "{}", s).ok();
                }
            },
            None => match result {
                Ok(s) => {
                    write!(io::stdout(), "{}", s).ok();

                    if !s.ends_with('\n') {
                        writeln!(io::stdout()).ok();
                    }
                }
                Err(s) => {
                    write!(io::stderr(), "{}", s).ok();

                    if !s.ends_with('\n') {
                        writeln!(io::stderr()).ok();
                    }
                }
            },
        }
    }
}
