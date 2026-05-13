use anyhow::{Ok, Result};

use crate::cmd::context::Context;

pub struct History;

impl History {
    pub fn handle(ctx: &mut Context) -> Result<()> {
        let guard = ctx.state.lock().unwrap();

        let limit = ctx.args
            .get(0)
            .and_then(|s| s.parse::<usize>().ok());

        let history = &guard.history;
        let end = history.len().saturating_sub(1);

        let start = if let Some(n) = limit {
            end.saturating_sub(n)
        } else {
            0
        };

        for (i, cmd) in history[start..end].iter().enumerate() {
            ctx.stdout.writeln(&format!(
                "    {} {}",
                start + i + 1,
                cmd
            ))?;
        }

        Ok(())
    }
}