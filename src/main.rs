use std::io::{self, Write};

use console::Term;

use crate::{
    cmd::context::{CompletionStore, Context},
    utils::get_user_input,
};

mod cmd;
mod utils;

fn main() {
    let mut completions = CompletionStore::new();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut term = Term::stdout();

        let input = get_user_input(&mut term, &mut completions).unwrap();

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
