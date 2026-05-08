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
