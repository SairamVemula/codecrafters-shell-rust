use anyhow::{Ok, Result};

use crate::cmd::context::Context;

pub struct History;

impl History {
    pub fn handle(ctx: &mut Context) -> Result<()> {
        let guard = ctx.state.lock().unwrap();
        for (i, cmd) in guard.history.iter().enumerate() {
            ctx.stdout.writeln(&format!("    {}) {}", i + 1, cmd))?;
        }
        Ok(())
    }
}
