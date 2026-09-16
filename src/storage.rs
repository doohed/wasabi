use std::path::PathBuf;

/// Directory this app keeps its files in: `$XDG_DATA_HOME/wasabi`, falling
/// back to `~/.local/share/wasabi`.
///
/// `None` when neither variable is set, which every caller has to treat as
/// "this session has nowhere to persist to" rather than as an error.
pub fn data_dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
        })?;

    Some(base.join("wasabi"))
}

/// A path with the home directory written as `~`, for display only.
pub fn tilde(path: &std::path::Path) -> String {
    let home = std::env::var_os("HOME").map(PathBuf::from);

    match home.and_then(|home| path.strip_prefix(home).ok().map(PathBuf::from)) {
        Some(rest) => format!("~/{}", rest.display()),
        None => path.display().to_string(),
    }
}
