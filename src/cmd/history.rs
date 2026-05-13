use anyhow::{Ok, Result};

use crate::cmd::context::Context;

pub struct History;
impl History {
    pub fn handle(ctx: &mut Context) -> Result<()> {
        let guard = ctx.state.lock().unwrap();

        let limit = ctx.args.get(0)
            .and_then(|s| s.parse::<usize>().ok());

        let iter = guard.history.iter();

        if let Some(n) = limit {
            for (i, cmd) in iter.enumerate().rev().take(n) {
                ctx.stdout.writeln(&format!("    {} {}", i + 1, cmd))?;
            }
        } else {
            for (i, cmd) in iter.enumerate() {
                ctx.stdout.writeln(&format!("    {} {}", i + 1, cmd))?;
            }
        }

        Ok(())
    }
}