use std::{
    fs::OpenOptions,
    io::{self, Write},
};

use console::Term;

use crate::{
    cmd::context::{Context, OutputDestination},
    utils::get_user_input,
};

mod cmd;
mod utils;

fn main() {
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

        let mut ctx = Context::new(parsed);

        for redir in redirections {
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(matches!(
                    redir.r_type,
                    utils::StdRedirectionType::StdoutAppend
                        | utils::StdRedirectionType::StderrAppend
                ))
                .truncate(!matches!(
                    redir.r_type,
                    utils::StdRedirectionType::StdoutAppend
                        | utils::StdRedirectionType::StderrAppend
                ))
                .open(&redir.file)
                .expect("Failed to open redirection file");

            match redir.r_type {
                utils::StdRedirectionType::StdoutWrite
                | utils::StdRedirectionType::StdoutAppend => {
                    ctx.stdout = OutputDestination::File(file);
                }
                utils::StdRedirectionType::StderrWrite
                | utils::StdRedirectionType::StderrAppend => {
                    ctx.stderr = OutputDestination::File(file);
                }
            }
        }

        let result = cmd::dispatch(&cmd_name, &mut ctx);

        if let Err(e) = result {
            if !e.is_empty() {
                ctx.stderr.writeln(&e).ok();
            }
        }
    }
}
