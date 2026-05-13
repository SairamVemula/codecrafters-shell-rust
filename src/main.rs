use std::io::{self, Write};

use crate::cmd::{
    context::{AppState, Context},
    jobs::Job,
};

mod cmd;
mod utils;

fn main() {
    let state = AppState::new();
    loop {
        Job::check_jobs(state.clone()).unwrap();
        print!("$ ");
        io::stdout().flush().unwrap();

        match Context::new(state.clone(), None) {
            Ok(ctx) => execute(ctx),
            Err(_) => continue,
        }
    }
}

fn execute(mut ctx: Context) {
    let result = cmd::dispatch(&mut ctx);

    if let Err(e) = result {
        ctx.stderr.writeln(&format!("{e}")).ok();
    }

    for pipe in ctx.pipes {
        execute(pipe);
    }
}
