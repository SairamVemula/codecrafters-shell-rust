use crate::cmd::context::Context;

pub fn handle_complete(ctx: &mut Context) -> Result<(), String> {
    ctx.stderr
        .writeln(&format!(
            "complete: {}: no completion specification",
            ctx.args.last().unwrap_or(&"nothing".to_string())
        ))
        .map_err(|e| e.to_string())
}
