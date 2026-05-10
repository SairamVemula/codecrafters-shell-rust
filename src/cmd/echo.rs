use anyhow::Result;
use crate::cmd::context::Context;

pub fn handle_echo(ctx: &mut Context) -> Result<()> {
    ctx.stdout.writeln(&ctx.args.join(" "))?;
    Ok(())
}
