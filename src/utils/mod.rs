use std::{
    collections::BTreeSet,
    env, fs,
    io::{self, Write},
    ops::Bound::{Included, Unbounded},
    path::{self, MAIN_SEPARATOR, Path, PathBuf},
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
                let (clear_len, possible_autocompletes) = autocomplete(&input);
                if possible_autocompletes.len() == 1
                    && let Some(autocomplete) = possible_autocompletes.first()
                {
                    input.clear_chars(clear_len);
                    input.push_str(&format!("{autocomplete}"));
                    term.clear_chars(clear_len)?;
                    print!("{autocomplete}");
                } else if possible_autocompletes.len() > 1
                    && let Some(autocomplete) =
                        longest_comman_prefix_from_btreeset(&possible_autocompletes, &input)
                {
                    input.clear_chars(clear_len);
                    input.push_str(&format!("{autocomplete}"));
                    term.clear_chars(clear_len)?;
                    print!("{autocomplete}");
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
                                .into_iter()
                                .collect::<Vec<String>>()
                                .join(" ")
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
        .into_iter()
        .map(|mut f| {
            f.push(' ');
            f
        })
        .collect()
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
        if &lcp != prefix {
            return Some(lcp);
        }
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

pub fn autocomplete(input: &str) -> (usize, BTreeSet<String>) {
    let args = parse_args(input);
    if args.len() > 1 {
        return find_files_or_dirs(&args[1]);
    }
    (args[0].len(), find_possible_command(&args[0]))
}
