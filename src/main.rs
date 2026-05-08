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
            Command::Type(args) => {
                println!("{}", cmd::handle_type(args));
            }
            _ => {
                println!("{}: command not found", input.trim())
            }
        }
    }
}
