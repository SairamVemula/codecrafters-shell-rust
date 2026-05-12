use std::io::{self, Write};

use crate::cmd::context::{Context, Store};

mod cmd;
mod utils;

fn main() {
    let mut store = Store::new();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        match Context::new(&mut store) {
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
