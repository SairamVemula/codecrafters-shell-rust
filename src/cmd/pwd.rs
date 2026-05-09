use std::env;
use crate::cmd::context::Context;

pub fn handle_pwd(ctx: &mut Context) -> Result<(), String> {
    ctx.stdout.writeln(&format!("{}", env::current_dir().unwrap().display())).map_err(|e| e.to_string())
}
