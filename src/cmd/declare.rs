use anyhow::{Result, anyhow};

use crate::cmd::context::{Context, SharedState};

pub enum DeclareFlag {
    Print,
    Input,
}

pub struct Declare {
    flag: DeclareFlag,
    args: Vec<String>,
}

impl Declare {
    pub fn parse(args: Vec<String>) -> Self {
        match args[0].as_str() {
            "-p" => Declare {
                flag: DeclareFlag::Print,
                args: args[1..].to_vec(),
            },
            _ => Self {
                flag: DeclareFlag::Input,
                args,
            },
        }
    }
    pub fn handle(ctx: &mut Context) -> Result<()> {
        // if ctx.args.len() == 0 {

        // }
        let mut d = Declare::parse(ctx.args.clone());

        match d.flag {
            DeclareFlag::Print => d.print(ctx),
            DeclareFlag::Input => d.save(ctx),
        }
    }

    pub fn save(&mut self, ctx: &mut Context) -> Result<()> {
        let mut guard = ctx.state.lock().unwrap();
        for arg in self.args.clone() {
            let s: Vec<&str> = arg.split('=').collect();
            if s.len() == 2 && is_valid_variable_name(s[0]) {
                guard.variables.insert(s[0].to_string(), s[1].to_string());
            } else {
                ctx.stderr
                    .writeln(&format!("declare: `{arg}': not a valid identifier"))
                    .unwrap()
            }
        }
        Ok(())
    }

    pub fn print(&mut self, ctx: &mut Context) -> Result<()> {
        let guard = ctx.state.lock().unwrap();
        for arg in self.args.clone() {
            match guard.variables.get(&arg) {
                Some(v) => ctx
                    .stdout
                    .writeln(&format!("declare -- {}=\"{}\"", arg, v))
                    .unwrap(),
                None => ctx
                    .stderr
                    .writeln(&format!("declare: {}: not found", arg))
                    .unwrap(),
            }
        }
        Ok(())
    }
}

fn is_valid_variable_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();

    let is_valid = match chars.next() {
        Some(c) => c.is_alphabetic() || c == '_',
        _ => false,
    };

    if !is_valid {
        return false;
    }

    chars.all(|c| c.is_alphanumeric() || c == '_')
}
