#[allow(unused_imports)]
use std::io::{self, Write};

mod cmd;

fn main() {
    // TODO: Uncomment the code below to pass the first stage
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();

        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "exit" => {
                break;
            }
            _ => {
                if input.starts_with("echo ") {
                    let args: Vec<&str> = input.trim()[5..].split_whitespace().collect();
                    cmd::echo::handle_echo(args);
                } else {
                    println!("{}: command not found", input.trim())
                }
            }
        }
    }
}
