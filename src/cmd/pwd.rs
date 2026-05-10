use anyhow::Result;
use crate::cmd::context::Context;
use std::env;

pub fn handle_pwd(ctx: &mut Context) -> Result<()> {
    ctx.stdout.writeln(&format!("{}", env::current_dir().unwrap().display()))?;
    Ok(())
}
