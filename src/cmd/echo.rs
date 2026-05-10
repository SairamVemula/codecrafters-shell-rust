use crate::cmd::context::Context;

pub fn handle_echo(ctx: &mut Context) -> Result<(), String> {
    ctx.stdout
        .writeln(&ctx.args.join(" "))
        .map_err(|e| e.to_string())
}
