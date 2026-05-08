use std::{env, path::Path};

pub fn handle_cd(args: Vec<&str>) {
    let path = Path::new(args[0]);

    if let Err(_) = env::set_current_dir(path) {
        println!("cd: {}: : No such file or directory", path.display());

    }
}
