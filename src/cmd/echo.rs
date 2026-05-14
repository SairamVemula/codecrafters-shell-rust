use crate::cmd::{context::Context, declare};
use anyhow::Result;

pub fn handle_echo(ctx: &mut Context) -> Result<()> {
    let args = declare::replace_variables(ctx.args.clone(), &ctx.state);
    ctx.stdout.writeln(&args.join(" "))?;
    Ok(())
}
