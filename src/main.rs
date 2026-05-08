use std::io::{self, Write};

use crate::cmd::Command;

mod cmd;
mod utils;

fn main() {
    // TODO: Uncomment the code below to pass the first stage
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();

        io::stdin().read_line(&mut input).unwrap();

        let command = Command::from_raw(&input);

        match command {
            Command::Exit => {
                break;
            }
            Command::Echo(args) => {
                cmd::echo::handle_echo(args);
            }
            Command::Type(args) => match cmd::handle_type(args) {
                cmd::Type::Exe(cmd, path) => println!("{} is {}", cmd, path),
                cmd::Type::Unknown(cmd) => println!("{}: not found", cmd),
                cmd::Type::BuiltIn(cmd) => println!("{} is a shell builtin", cmd),
            },
            Command::Pwd(args) => {
                cmd::pwd::handle_pwd(args);
            },
            Command::Unknown(cmd, args) => {
                cmd::handle_run(cmd, args);
            }
        }
    }
}
