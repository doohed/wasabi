use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::config::storage;
use crate::typing::mode::{Language, Mode};
use crate::typing::modifiers::Modifiers;

const FILE: &str = "settings.tsv";
const THEME: &str = "theme";
const DURATION: &str = "duration";
const BANNER: &str = "banner";
const PUNCTUATION: &str = "punctuation";
const NUMBERS: &str = "numbers";
const CODE: &str = "code";

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
            .map_or(crate::config::theme::DEFAULT, String::as_str)
    }

    pub fn set_theme(&mut self, name: &str) {
        self.values.insert(THEME.to_string(), name.to_string());
        self.save();
    }

    /// The stored test length in seconds, if there is a usable one.
    ///
    /// `Option` rather than a default: which lengths are legal is the app's
    /// business, not this file's, so the caller decides what to do with a
    /// missing or unparseable value.
    pub fn duration(&self) -> Option<u64> {
        self.values.get(DURATION)?.parse().ok()
    }

    pub fn set_duration(&mut self, seconds: u64) {
        self.values
            .insert(DURATION.to_string(), seconds.to_string());
        self.save();
    }

    /// Whether the art above the test should be drawn at all.
    ///
    /// Defaults to on, and anything unrecognised reads as on: a typo in a
    /// hand-edited file shouldn't make the art quietly disappear.
    pub fn banner_shown(&self) -> bool {
        !matches!(
            self.values.get(BANNER).map(String::as_str),
            Some("off" | "false" | "no" | "0")
        )
    }

    pub fn set_banner_shown(&mut self, shown: bool) {
        let value = if shown { "on" } else { "off" };
        self.values.insert(BANNER.to_string(), value.to_string());
        self.save();
    }

    /// What the test does to its words once they're dealt.
    ///
    /// Both default to off, and anything unrecognised reads as off: a typo in
    /// a hand-edited file shouldn't quietly change what the test is.
    pub fn modifiers(&self) -> Modifiers {
        Modifiers {
            punctuation: self.flag(PUNCTUATION),
            numbers: self.flag(NUMBERS),
        }
    }

    pub fn set_modifiers(&mut self, modifiers: Modifiers) {
        self.set_flag(PUNCTUATION, modifiers.punctuation);
        self.set_flag(NUMBERS, modifiers.numbers);
        self.save();
    }

    /// What kind of test this is.
    ///
    /// Composed rather than stored whole: the word settings stay on disk while
    /// a code test is being typed, so switching back finds them as you left
    /// them. Anything unrecognised in the `code` key reads as the word test,
    /// which is what a typo in a hand-edited file should cost you.
    pub fn mode(&self) -> Mode {
        match self.language() {
            Some(language) => Mode::Code(language),
            None => Mode::Words(self.modifiers()),
        }
    }

    fn language(&self) -> Option<Language> {
        Language::from_slug(self.values.get(CODE)?)
    }

    /// Turn the code test on for a language, or off with `None`.
    pub fn set_language(&mut self, language: Option<Language>) {
        let value = language.map_or("off", Language::slug);
        self.values.insert(CODE.to_string(), value.to_string());
        self.save();
    }

    fn flag(&self, key: &str) -> bool {
        matches!(
            self.values.get(key).map(String::as_str),
            Some("on" | "true" | "yes" | "1")
        )
    }

    fn set_flag(&mut self, key: &str, on: bool) {
        let value = if on { "on" } else { "off" };
        self.values.insert(key.to_string(), value.to_string());
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
    Some(storage::config_dir()?.join(FILE))
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
        assert_eq!(Settings::detached().theme(), crate::config::theme::DEFAULT);
    }

    #[test]
    fn a_theme_reads_back_after_being_set() {
        let mut settings = Settings::detached();
        settings.set_theme("nord");

        assert_eq!(settings.theme(), "nord");
    }

    #[test]
    fn a_duration_reads_back_after_being_set() {
        let mut settings = Settings::detached();
        assert_eq!(settings.duration(), None);

        settings.set_duration(15);
        assert_eq!(settings.duration(), Some(15));
    }

    #[test]
    fn a_duration_that_isnt_a_number_reads_as_absent() {
        let mut settings = Settings::detached();
        settings
            .values
            .insert(DURATION.to_string(), "ages".to_string());

        assert_eq!(settings.duration(), None);
    }

    #[test]
    fn the_banner_is_shown_unless_turned_off() {
        let mut settings = Settings::detached();
        assert!(settings.banner_shown());

        settings.set_banner_shown(false);
        assert!(!settings.banner_shown());

        settings.set_banner_shown(true);
        assert!(settings.banner_shown());
    }

    #[test]
    fn an_unrecognised_banner_value_leaves_it_on() {
        let mut settings = Settings::detached();
        settings
            .values
            .insert(BANNER.to_string(), "maybe".to_string());

        assert!(settings.banner_shown());
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
