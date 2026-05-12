use std::io::{self, Write};

use crate::cmd::context::{Context, AppState};

mod cmd;
mod utils;

fn main() {
    let state = AppState::new();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        match Context::new(state.clone()) {
            Ok(mut ctx) => {
                let result = cmd::dispatch(&mut ctx);

                if let Err(e) = result {
                    ctx.stderr.writeln(&format!("{e}")).ok();
                }
            }
            Err(_) => continue,
        }
    }
}
