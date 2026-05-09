use std::env;

pub fn handle_pwd(_: Vec<String>) -> Result<String, String> {
    Ok(format!("{}", env::current_dir().unwrap().display()))
}
