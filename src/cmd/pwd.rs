use std::env;

pub fn handle_pwd(_: Vec<&str>) {
    println!("{}", env::current_dir().unwrap().display())
}
