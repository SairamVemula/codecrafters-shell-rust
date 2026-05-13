use std::{
    collections::VecDeque,
    io::{self, Write},
    thread,
};

use crate::cmd::{
    context::{AppState, Context}, history::History, jobs::Job
};

mod cmd;
mod utils;

fn main() {
    let mut state = AppState::new();
    // if let Some(mut history) = History::new() {
    //     let _ = history.load(&mut state);
    // }
    loop {
        Job::check_jobs(state.clone()).unwrap();
        print!("$ ");
        io::stdout().flush().unwrap();

        match Context::new(state.clone(), None) {
            Ok(ctx) => {
                execute(ctx)
            },
            Err(_) => continue,
        }
    }
}
fn execute(mut ctx: Context) {
    let mut pipes = VecDeque::from(std::mem::take(&mut ctx.pipes));

    pipes.push_front(ctx);

    if let Some(last_pipe) = pipes.pop_back() {
        for pipe in pipes {
            thread::spawn(move || execute(pipe));
        }
        let mut final_ctx = last_pipe;
        if let Err(e) = cmd::dispatch(&mut final_ctx) {
            let _ = final_ctx.stderr.writeln(&format!("{e}"));
        }
    }
}
