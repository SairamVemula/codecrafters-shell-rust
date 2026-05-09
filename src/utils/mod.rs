use std::path;

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
                '\\' => {
                    if let Some(next) = chars.next() {
                        match next {
                            '\'' | '\\' => {
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
