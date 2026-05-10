use std::{
    collections::HashMap, io::{self, Write}
};

use console::Term;

use crate::{
    cmd::context::Context,
    utils::get_user_input,
};

mod cmd;
mod utils;

fn main() {
    let mut completions = HashMap::new();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let term = Term::stdout();

        let input = get_user_input(term).unwrap();

        let mut parsed = utils::parse_args(input.trim());

        if parsed.is_empty() {
            continue;
        }

        let cmd_name = parsed.remove(0);

        let redirections = utils::parse_redirections(&mut parsed);

        let mut ctx = Context::new(parsed, &mut completions);

        if let Err(e) = ctx.apply_redirections(redirections) {
            eprintln!("Error applying redirections: {e}");
            continue;
        }

        let result = cmd::dispatch(&cmd_name, &mut ctx);

        if let Err(e) = result {
            ctx.stderr.writeln(&format!("{e}")).ok();
        }
    }
}
