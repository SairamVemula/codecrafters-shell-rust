use anyhow::Result;

use crate::cmd::context::Context;

pub struct Jobs;

impl Jobs {
    pub fn handle(_ctx: &mut Context) -> Result<()> {
        Ok(())
    }
}
