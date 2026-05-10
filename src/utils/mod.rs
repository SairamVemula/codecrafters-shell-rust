use std::{
    collections::BTreeSet,
    env, fs,
    io::{self, Write},
    ops::Bound::{Included, Unbounded},
    path,
};

use anyhow::{Context, Result};
use console::Term;

use crate::cmd::BuiltInCommand;

#[cfg(windows)]
pub fn is_executable(path: &path::Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|ext| matches!(ext, "exe" | "bat" | "cmd"))
        .unwrap_or(false)
}

#[cfg(unix)]
pub fn is_executable(path: &path::Path) -> bool {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    if let Ok(meta) = fs::metadata(path) {
        meta.is_file() && (meta.permissions().mode() & 0o111 != 0)
    } else {
        false
    }
}

#[derive(Debug, Clone, Copy)]
enum ArgParserState {
    Normal,
    SingleQuote,
    DoubleQuote,
}

pub fn parse_args(input: &str) -> Vec<String> {
    let mut args = vec![];
    let mut current = String::new();

    let mut chars = input.chars().peekable();

    let mut state = ArgParserState::Normal;

    while let Some(ch) = chars.next() {
        match state {
            ArgParserState::Normal => match ch {
                '\'' => {
                    state = ArgParserState::SingleQuote;
                }
                '"' => {
                    state = ArgParserState::DoubleQuote;
                }
                '\\' => {
                    if let Some(next) = chars.next() {
                        current.push(next);
                    }
                }
                c if c.is_whitespace() => {
                    if !current.is_empty() {
                        args.push(std::mem::take(&mut current));
                    }
                }
                _ => {
                    current.push(ch);
                }
            },
            ArgParserState::SingleQuote => match ch {
                '\'' => {
                    state = ArgParserState::Normal;
                }

                _ => {
                    current.push(ch);
                }
            },
            ArgParserState::DoubleQuote => match ch {
                '"' => {
                    state = ArgParserState::Normal;
                }
                '\\' => {
                    if let Some(next) = chars.next() {
                        match next {
                            '"' | '\\' => {
                                current.push(next);
                            }
                            other => {
                                current.push(ch);
                                current.push(other);
                            }
                        }
                    }
                }
                _ => {
                    current.push(ch);
                }
            },
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

#[derive(Debug, PartialEq)]
pub enum StdRedirectionType {
    StdoutWrite,
    StdoutAppend,
    StderrWrite,
    StderrAppend,
}

#[derive(Debug)]
pub struct Redirection {
    pub r_type: StdRedirectionType,
    pub file: String,
}

pub fn parse_redirections(args: &mut Vec<String>) -> Vec<Redirection> {
    let mut redirections = vec![];
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        let redir_type = match arg.as_str() {
            ">" | "1>" => Some(StdRedirectionType::StdoutWrite),
            ">>" | "1>>" => Some(StdRedirectionType::StdoutAppend),
            "2>" => Some(StdRedirectionType::StderrWrite),
            "2>>" => Some(StdRedirectionType::StderrAppend),
            _ => None,
        };

        if let Some(r_type) = redir_type {
            if i + 1 < args.len() {
                let file = args.remove(i + 1);
                args.remove(i);
                redirections.push(Redirection { r_type, file });
                continue;
            }
        }
        i += 1;
    }
    redirections
}

pub fn get_user_input(term: Term) -> Result<String> {
    let mut input = String::new();
    let mut bell_rang = false;
    loop {
        let key = term.read_key().context("Reading each key")?;
        match key {
            console::Key::Unknown => todo!(),
            console::Key::UnknownEscSeq(_) => todo!(),
            console::Key::ArrowLeft => todo!(),
            console::Key::ArrowRight => todo!(),
            console::Key::ArrowUp => todo!(),
            console::Key::ArrowDown => todo!(),
            console::Key::Enter => break,
            console::Key::Escape => todo!(),
            console::Key::Backspace => {
                input.pop();
                term.clear_line()?;
                print!("$ {input}")
            }
            console::Key::Home => todo!(),
            console::Key::End => todo!(),
            console::Key::Tab => {
                let possible_cmds = find_possible_command(&input);
                if possible_cmds.len() == 1
                    && let Some(cmd) = possible_cmds.first()
                {
                    input.clear();
                    input.push_str(&format!("{cmd} "));
                    term.clear_line()?;
                    print!("$ {cmd} ");
                } else if possible_cmds.len() > 1
                    && let Some(cmd) = longest_comman_prefix_from_btreeset(&possible_cmds, &input)
                {
                    input.clear();
                    input.push_str(&format!("{cmd} "));
                    term.clear_line()?;
                    print!("$ {cmd}");
                } else {
                    if !bell_rang {
                        bell_rang = true;
                        print!("\x07");
                    } else {
                        bell_rang = false;
                        println!();
                        println!(
                            "{}",
                            possible_cmds
                                .into_iter()
                                .collect::<Vec<String>>()
                                .join("  ")
                        );
                        print!("$ {input}");
                    }
                }
            }
            console::Key::BackTab => todo!(),
            console::Key::Alt => todo!(),
            console::Key::Del => todo!(),
            console::Key::Shift => {}
            console::Key::Insert => todo!(),
            console::Key::PageUp => todo!(),
            console::Key::PageDown => todo!(),
            console::Key::Char(ch) => {
                input.push(ch);
                print!("{ch}");
            }
            console::Key::CtrlC => todo!(),
            _ => todo!(),
        };
        io::stdout().flush()?;
    }
    println!();
    Ok(input)
}

pub fn find_possible_path_to_command(cmd: &String) -> Option<String> {
    let path = env::var("PATH").unwrap();
    for dir in env::split_paths(&path) {
        let base_path = dir.join(cmd);
        if base_path.exists() && is_executable(&base_path) {
            return Some(base_path.display().to_string());
        }
    }
    None
}
pub fn find_posible_path_command(prefix: &str) -> BTreeSet<String> {
    let mut commands = BTreeSet::new();
    let path = env::var("PATH").unwrap();
    for dir in env::split_paths(&path) {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        for entry in entries {
            let Ok(entry) = entry else {
                continue;
            };
            let file_name = entry.file_name();
            if file_name.to_str().is_some_and(|f| f.starts_with(prefix)) {
                if let Ok(name) = file_name.into_string() {
                    commands.insert(name);
                }
            }
        }
    }
    commands
}

pub fn find_possible_command(prefix: &str) -> BTreeSet<String> {
    let mut builtin_matches = BuiltInCommand::matches(prefix);
    let path_cmd_matches = find_posible_path_command(prefix);
    builtin_matches.extend(path_cmd_matches);

    builtin_matches
}

pub fn longest_comman_prefix_from_btreeset(
    btree: &BTreeSet<String>,
    prefix: &String,
) -> Option<String> {
    let mut subset = btree
        .range::<str, _>((Included(prefix.as_str()), Unbounded))
        .take_while(|s| s.starts_with(prefix))
        .peekable();

    let first = subset.peek().copied();
    let last = subset.last();

    if let (Some(first), Some(last)) = (first, last) {
        let lcp: String = first
            .chars()
            .zip(last.chars())
            .take_while(|(f, l)| f == l)
            .map(|(f, _)| f)
            .collect();
        return Some(lcp);
    }

    None
}
