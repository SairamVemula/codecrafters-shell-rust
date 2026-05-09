use std::env;

pub fn handle_pwd(_: Vec<String>) {
    println!("{}", env::current_dir().unwrap().display())
}
