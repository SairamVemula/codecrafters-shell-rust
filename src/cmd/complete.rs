use anyhow::{anyhow, Result};
use std::path::PathBuf;

use crate::cmd::context::Context;

#[derive(Debug)]
pub enum CompleteFlag {
    Print,
    Register,
}
#[derive(Debug)]
pub struct Complete {
    flag: CompleteFlag,
    path: Option<PathBuf>,
    command: String,
}

impl From<&Vec<String>> for Complete {
    fn from(value: &Vec<String>) -> Self {
        match value[0].as_str() {
            "-C" => Self {
                flag: CompleteFlag::Register,
                path: Some(PathBuf::from(value[1].clone())),
                command: value[2].clone(),
            },

            "-p" => Self {
                flag: CompleteFlag::Print,
                path: None,
                command: value[1].clone(),
            },

            _ => todo!(),
        }
    }
}

impl Complete {
    pub fn handle(ctx: &mut Context) -> Result<()> {
        let complete = Complete::from(&ctx.args);
        
        let result = match complete.flag {
            CompleteFlag::Print => match ctx.completions.get(&complete.command) {
                Some(path) => Ok(format!(
                    "complete -C '{}' {}",
                    path.display(),
                    complete.command
                )),

                None => Err(anyhow!(
                    "complete: {}: no completion specification",
                    complete.command
                )),
            },

            CompleteFlag::Register => match complete.path {
                Some(path) => {
                    ctx.completions.insert(complete.command, path);
                    Ok(String::new())
                }

                None => Err(anyhow!("complete: missing path")),
            },
        };

        match result {
            Ok(output) => {
                if !output.is_empty() {
                    ctx.stdout.writeln(&output)?;
                }
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}
