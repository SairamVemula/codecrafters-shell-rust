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
                    println!("{}", &input[5..]);
                } else {
                    println!("{}: command not found", input.trim())
                }
            }
        }
    }
}
