use std::{env, path::PathBuf};
use crate::cmd::context::Context;

pub fn handle_cd(ctx: &mut Context) -> Result<(), String> {
    let path = ctx.args.get(0).map_or("~", |v| v);

    let expanded = expand_path(path);

    if let Err(_) = env::set_current_dir(&expanded) {
        return Err(format!(
            "cd: {}: No such file or directory",
            expanded.display()
        ));
    }
    Ok(())
}

fn expand_path(input: &str) -> PathBuf {
    if input == "~" {
        return env::var("HOME")
            .or_else(|_| env::var("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/"));
    }

    if let Some(stripped) = input.strip_prefix("~/") {
        let mut home = env::var("HOME")
            .or_else(|_| env::var("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/"));

        home.push(stripped);
        return home;
    }

    PathBuf::from(input)
}
