use std::{
    collections::BTreeSet,
    env, fs,
    io::{self, Write},
    ops::Bound::{Included, Unbounded},
    path::{self, MAIN_SEPARATOR, Path, PathBuf},
};

use anyhow::{Context, Result};
use console::Term;

use crate::cmd::context::{Redirection, RedirectionType};
use crate::cmd::{BuiltInCommand, complete::Complete, context::CompletionStore};

#[cfg(windows)]
pub fn is_executable(path: &path::Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|ext| matches!(ext, "exe" | "bat" | "cmd"))
        .unwrap_or(false)
}

#[cfg(unix)]
pub fn is_executable(path: &path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    if let Ok(meta) = fs::metadata(path) {
        meta.is_file() && (meta.permissions().mode() & 0o111 != 0)
    } else {
        false
    }
}

trait StringExt {
    fn clear_chars(&mut self, n: usize);
}

impl StringExt for String {
    fn clear_chars(&mut self, n: usize) {
        let current_char_count = self.chars().count();
        let keep_count = current_char_count.saturating_sub(n);
        let byte_index = self
            .char_indices()
            .nth(keep_count)
            .map(|(idx, _)| idx)
            .unwrap_or(self.len());
        self.truncate(byte_index);
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

    if !current.is_empty() || input.ends_with(' ') {
        args.push(current);
    }

    args
}

pub fn unparse_args(args: &[String]) -> String {
    args.iter()
        .map(|arg| {
            if arg.is_empty() {
                return String::from("\"\"");
            }

            let needs_quotes = arg
                .chars()
                .any(|c| c.is_whitespace() || c == '\'' || c == '"' || c == '\\');

            if !needs_quotes {
                arg.clone()
            } else {
                let mut escaped = String::from("\"");
                for ch in arg.chars() {
                    match ch {
                        '"' | '\\' => {
                            escaped.push('\\');
                            escaped.push(ch);
                        }
                        _ => escaped.push(ch),
                    }
                }
                escaped.push('"');
                escaped
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

pub fn parse_redirections(args: &mut Vec<String>) -> (Vec<Vec<String>>, Vec<Redirection>) {
    let (pipes, mut redirections) = parse_pipes(args);
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        let redir_type = match arg.as_str() {
            ">" | "1>" => Some(RedirectionType::StdoutWrite),
            ">>" | "1>>" => Some(RedirectionType::StdoutAppend),
            "2>" => Some(RedirectionType::StderrWrite),
            "2>>" => Some(RedirectionType::StderrAppend),
            _ => None,
        };

        if let Some(r_type) = redir_type {
            if i + 1 < args.len() {
                let file = args.remove(i + 1);
                args.remove(i);
                redirections.push(Redirection {
                    r_type,
                    file: Some(file),
                    pipe_reader: None,
                    pipe_writer: None,
                });
                continue;
            }
        }
        i += 1;
    }
    (pipes, redirections)
}
pub fn parse_pipes(args: &mut Vec<String>) -> (Vec<Vec<String>>, Vec<Redirection>) {
    let mut redirections = vec![];
    let mut pipes = vec![];
    let mut pipe = vec![];
    let mut i = 0;
    let mut is_pipe = false;
    let mut pipe_reader = None;

    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "|" => {
                let (reader, write) = io::pipe().unwrap();
                redirections.push(Redirection {
                    r_type: RedirectionType::StdoutPipe,
                    file: None,
                    pipe_reader: pipe_reader.take(),
                    pipe_writer: Some(write),
                });
                pipe_reader = Some(reader);
                if is_pipe {
                    pipes.push(std::mem::take(&mut pipe));
                }
                is_pipe = true;
                args.remove(i);
                continue;
            }
            "&" | ">" | "1>" | ">>" | "1>>" | "2>" | "2>>" => {
                redirections.push(Redirection {
                    r_type: RedirectionType::StdinPipe,
                    file: None,
                    pipe_reader: pipe_reader.take(),
                    pipe_writer: None,
                });
                break;
            }
            _ => {
                if is_pipe {
                    pipe.push(arg.clone());
                    args.remove(i);
                    continue;
                }
            }
        };
        i += 1;
    }

    if pipe.len() > 0 {
        redirections.push(Redirection {
            r_type: RedirectionType::StdinPipe,
            file: None,
            pipe_reader: pipe_reader.take(),
            pipe_writer: None,
        });
        pipes.push(pipe);
    }

    (pipes, redirections)
}

pub fn get_user_input(mut term: &mut Term, completions: &mut CompletionStore) -> Result<String> {
    let mut input = String::new();
    let mut bell_rang = false;
    loop {
        let key = term.read_key().context("Reading each key")?;
        match key {
            console::Key::Enter => break,
            console::Key::Backspace => {
                input.pop();
                term.clear_line()?;
                print!("$ {input}")
            }
            console::Key::Tab => {
                bell_rang = handle_tab(&mut input, &mut term, bell_rang, completions)?;
            }
            console::Key::Char(ch) => {
                input.push(ch);
                print!("{ch}");
            }
            console::Key::CtrlC => std::process::exit(0),
            _ => {}
        };
        io::stdout().flush()?;
    }
    println!();
    Ok(input)
}

fn handle_tab(
    input: &mut String,
    term: &mut Term,
    mut bell_rang: bool,
    completions: &mut CompletionStore,
) -> Result<bool> {
    let args = parse_args(input);
    let (clear_len, possible_autocompletes) = match args.len() > 1 {
        true => {
            //Complete Tab Handler
            if let Ok(suggestions) = Complete::autocomplete(&input, &args, completions) {
                suggestions
            } else {
                //Builtin Tab Handler
                let empty = String::new();
                let last = args.last().unwrap_or(&empty);
                find_files_or_dirs(last)
            }
        }
        false => (args[0].len(), find_possible_command(&args[0])),
    };
    if possible_autocompletes.len() == 1 {
        if let Some(autocomplete) = possible_autocompletes.first() {
            input.clear_chars(clear_len);
            input.push_str(autocomplete);
            term.clear_chars(clear_len)?;
            print!("{autocomplete}");
        }
    } else if possible_autocompletes.len() > 1 {
        if let Some(lcp) = longest_common_prefix_from_btreeset(&possible_autocompletes, input) {
            input.clear_chars(clear_len);
            input.push_str(&lcp);
            term.clear_chars(clear_len)?;
            print!("{lcp}");
        } else {
            if !bell_rang {
                bell_rang = true;
                print!("\x07");
            } else {
                bell_rang = false;
                println!();
                println!(
                    "{}",
                    possible_autocompletes
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(" ")
                );
                print!("$ {input}");
            }
        }
    } else {
        print!("\x07");
    }
    Ok(bell_rang)
}

pub fn find_possible_path_to_command(cmd: &str) -> Option<String> {
    let path = env::var("PATH").ok()?;
    for dir in env::split_paths(&path) {
        let base_path = dir.join(cmd);
        if base_path.exists() && is_executable(&base_path) {
            return Some(base_path.display().to_string());
        }
    }
    None
}

pub fn find_possible_path_command(prefix: &str) -> BTreeSet<String> {
    let mut commands = BTreeSet::new();
    let Ok(path) = env::var("PATH") else {
        return commands;
    };
    for dir in env::split_paths(&path) {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            if let Some(s) = file_name.to_str() {
                if s.starts_with(prefix) {
                    commands.insert(s.to_string());
                }
            }
        }
    }
    commands
}

pub fn find_possible_command(prefix: &str) -> BTreeSet<String> {
    let mut builtin_matches = BuiltInCommand::matches(prefix);
    let path_cmd_matches = find_possible_path_command(prefix);
    builtin_matches.extend(path_cmd_matches);

    builtin_matches
        .into_iter()
        .map(|mut f| {
            f.push(' ');
            f
        })
        .collect()
}

pub fn longest_common_prefix_from_btreeset(
    btree: &BTreeSet<String>,
    input: &str,
) -> Option<String> {
    let args = parse_args(input);
    let empty = "".to_string();
    let prefix = args.last().unwrap_or(&empty);

    let mut subset = btree
        .range::<str, _>((Included(prefix.as_str()), Unbounded))
        .take_while(|s| s.starts_with(prefix))
        .peekable();

    let first = subset.peek().copied()?;
    let last = subset.last()?;

    let lcp: String = first
        .chars()
        .zip(last.chars())
        .take_while(|(f, l)| f == l)
        .map(|(f, _)| f)
        .collect();

    if &lcp != prefix {
        return Some(lcp);
    }

    None
}

pub fn find_files_or_dirs(partial: &str) -> (usize, BTreeSet<String>) {
    let (curr_dir, search_str) = if partial.ends_with(MAIN_SEPARATOR) {
        (PathBuf::from(partial), "")
    } else {
        let path = Path::new(partial);
        let search = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let dir = match path.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
            _ => env::current_dir().unwrap(),
        };
        (dir, search)
    };

    let Ok(dir_list) = fs::read_dir(curr_dir) else {
        return (search_str.len(), BTreeSet::new());
    };

    let btree_set = dir_list
        .filter_map(|f| f.ok())
        .map(|f| {
            let mut name = f.file_name().to_string_lossy().into_owned();
            if f.file_type().unwrap().is_dir() {
                name.push(MAIN_SEPARATOR);
            } else {
                name.push(' ')
            }
            name
        })
        .filter(|name| name.starts_with(search_str))
        .collect();
    (search_str.len(), btree_set)
}
