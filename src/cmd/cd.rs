use std::{
    env,
    path::{ PathBuf},
};

pub fn handle_cd(args: Vec<&str>) {
    let path = args.get(0).unwrap_or(&"~");

    let expanded = expand_path(path);

    if let Err(_) = env::set_current_dir(&expanded) {
        println!("cd: {}: No such file or directory", expanded.display());
    }
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
