use crate::cmd::context::Context;
use anyhow::Result;

pub fn handle_echo(ctx: &mut Context) -> Result<()> {
    ctx.stdout.writeln(&ctx.args.join(" "))?;
    Ok(())
}
