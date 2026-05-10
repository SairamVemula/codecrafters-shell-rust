use anyhow::{Result, anyhow};
use std::{collections::BTreeSet, path::PathBuf};

use crate::cmd::context::{CompletionStore, Context};

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

    pub fn autocomplete(
        args: &Vec<String>,
        completions: &mut CompletionStore,
    ) -> Result<(usize, BTreeSet<String>)> {
        if args.len() < 2 {
            return Err(anyhow!("two arguments are required"));
        }
        let empty = String::new();
        let cmd: &String = &args.first().unwrap();
        let last: &String = &args.last().unwrap();
        let last_second: &String = &args.get(args.len() - 2).unwrap_or(&empty);
        if let Some(script_path) = completions.get(cmd) {
            let output = std::process::Command::new(script_path)
                .args(vec![cmd, last, last_second])
                .output()?;

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                return Ok((last.len(), stdout.lines().map(|s| format!("{s} ")).collect()));
            }
        }
        Err(anyhow!("complete: {}: no completion specification", cmd))
    }
}
