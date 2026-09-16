use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::storage;

const FILE: &str = "settings.tsv";
const THEME: &str = "theme";

/// Persisted preferences.
///
/// A key-value file rather than a struct of fields so that adding a preference
/// later doesn't invalidate everyone's existing file.
#[derive(Debug, Clone, Default)]
pub struct Settings {
    values: BTreeMap<String, String>,
    /// `None` for an in-memory instance, which makes [`Settings::save`] a
    /// no-op — that is what keeps tests off the user's real settings.
    path: Option<PathBuf>,
}

impl Settings {
    /// Read the settings file, or start with the defaults.
    ///
    /// Infallible, like the records and the banner: bad settings mean default
    /// settings, never a failure to start.
    pub fn load() -> Self {
        let path = settings_path();
        let text = path
            .as_ref()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .unwrap_or_default();

        Self {
            values: parse(&text),
            path,
        }
    }

    /// An in-memory instance, for tests.
    #[cfg(test)]
    pub fn detached() -> Self {
        Self::default()
    }

    pub fn theme(&self) -> &str {
        self.values
            .get(THEME)
            .map_or(crate::theme::DEFAULT, String::as_str)
    }

    pub fn set_theme(&mut self, name: &str) {
        self.values.insert(THEME.to_string(), name.to_string());
        self.save();
    }

    /// Best-effort write, for the same reason as the records: losing a
    /// preference is not worth interrupting a typing session over.
    fn save(&self) {
        let Some(path) = &self.path else {
            return; // in-memory instance
        };

        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, format(&self.values));
    }
}

fn settings_path() -> Option<PathBuf> {
    Some(storage::data_dir()?.join(FILE))
}

fn format(values: &BTreeMap<String, String>) -> String {
    values
        .iter()
        .map(|(key, value)| format!("{key}\t{value}\n"))
        .collect()
}

/// Parse `key<TAB>value` lines, skipping anything that doesn't fit.
///
/// Unknown keys are kept, not dropped: downgrading to an older build and back
/// shouldn't silently erase the settings the newer one wrote.
fn parse(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .filter_map(|line| line.split_once('\t'))
        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
        .filter(|(key, _)| !key.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_default_to_the_default_theme() {
        assert_eq!(Settings::detached().theme(), crate::theme::DEFAULT);
    }

    #[test]
    fn a_theme_reads_back_after_being_set() {
        let mut settings = Settings::detached();
        settings.set_theme("nord");

        assert_eq!(settings.theme(), "nord");
    }

    #[test]
    fn settings_survive_a_write_and_read() {
        let mut values = BTreeMap::new();
        values.insert(THEME.to_string(), "magenta".to_string());

        assert_eq!(parse(&format(&values)), values);
    }

    #[test]
    fn junk_lines_are_skipped_not_fatal() {
        let values = parse("no tab here\n\tempty key\ntheme\tnord\n");

        assert_eq!(values.len(), 1);
        assert_eq!(values[THEME], "nord");
    }

    #[test]
    fn unknown_keys_are_preserved() {
        let values = parse("something_new\t42\n");
        assert_eq!(format(&values), "something_new\t42\n");
    }

    #[test]
    fn an_in_memory_settings_never_touches_the_disk() {
        let mut settings = Settings::detached();
        settings.set_theme("nord");

        assert!(settings.path.is_none());
    }
}
