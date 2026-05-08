#[allow(unused_imports)]
use std::io::{self, Write};

use crate::cmd::Command;

mod cmd;

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
            Command::Type(args) => match Command::from_raw(args[0]) {
                Command::Unkown => println!("{}: not found", args[0]),
                _ => println!("{} is a shell builtin", args[0]),
            },
            _ => {
                println!("{}: command not found", input.trim())
            }
        }
    }
}
