//! What kind of test this is.
//!
//! The word test and the code test are different enough that letting both be
//! configured at once would make states that mean nothing — punctuation has no
//! say over a snippet that was written with its own. An enum rather than a set
//! of flags, so "punctuated code" can't be represented, let alone recorded.

use super::modifiers::Modifiers;

/// A language the code test can deal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    C,
    Rust,
}

impl Language {
    /// Every language on offer, in the order the picker lists them.
    pub const ALL: [Language; 2] = [Language::C, Language::Rust];

    /// What to call it on screen.
    pub fn name(self) -> &'static str {
        match self {
            Language::C => "C",
            Language::Rust => "Rust",
        }
    }

    /// What to call it in a file: lowercase, and stable.
    ///
    /// Written into the settings and into every record key, so renaming one
    /// would silently orphan the records typed under it.
    pub fn slug(self) -> &'static str {
        match self {
            Language::C => "c",
            Language::Rust => "rust",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|language| language.slug() == slug)
    }
}

/// What the test deals, and what is done to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Words from the pool, with whatever is being done to them.
    Words(Modifiers),
    /// A pre-written snippet, typed as it was written.
    Code(Language),
}

impl Mode {
    /// How a record at this setting is named, appended to the test length.
    ///
    /// A code test is a different test — the same argument the length makes,
    /// and a stronger one, since a snippet full of braces scores nothing like
    /// a page of common words. The plain word test's key is the bare number,
    /// which is what every records file written before any of this existed
    /// already contains.
    pub fn key(self, seconds: u64) -> String {
        match self {
            Mode::Words(modifiers) => modifiers.key(seconds),
            Mode::Code(language) => format!("{seconds}+code:{}", language.slug()),
        }
    }

    /// What to call this setting on screen, or `None` for the plain word test.
    pub fn label(self) -> Option<String> {
        match self {
            Mode::Words(modifiers) => modifiers.label(),
            Mode::Code(language) => Some(format!("{} code", language.name())),
        }
    }

    /// The language being typed, if this is a code test.
    pub fn language(self) -> Option<Language> {
        match self {
            Mode::Words(_) => None,
            Mode::Code(language) => Some(language),
        }
    }

    /// The word-test settings in force, which is none of them in code mode.
    ///
    /// `Option` rather than a default: "punctuation is off" and "punctuation
    /// doesn't apply" are different answers, and the menu draws them
    /// differently.
    pub fn modifiers(self) -> Option<Modifiers> {
        match self {
            Mode::Words(modifiers) => Some(modifiers),
            Mode::Code(_) => None,
        }
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Words(Modifiers::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_language_survives_a_trip_through_its_slug() {
        for language in Language::ALL {
            assert_eq!(Language::from_slug(language.slug()), Some(language));
        }
    }

    #[test]
    fn an_unknown_language_is_not_one() {
        assert_eq!(Language::from_slug("cobol"), None);
        assert_eq!(Language::from_slug(""), None);
    }

    #[test]
    fn the_plain_word_test_keeps_the_bare_number() {
        // Which is what every records file written before any of this holds.
        assert_eq!(Mode::default().key(30), "30");
        assert_eq!(Mode::default().label(), None);
    }

    #[test]
    fn a_code_test_is_filed_under_its_language() {
        let mode = Mode::Code(Language::Rust);

        assert_eq!(mode.key(30), "30+code:rust");
        assert_eq!(Mode::Code(Language::C).key(15), "15+code:c");
        assert_eq!(mode.label().unwrap(), "Rust code");
    }

    #[test]
    fn the_word_settings_dont_apply_to_code() {
        let punctuated = Mode::Words(Modifiers {
            punctuation: true,
            numbers: false,
        });

        assert!(punctuated.modifiers().unwrap().punctuation);
        assert_eq!(punctuated.language(), None);

        // Not "off" — they have no say here at all, and the menu says so.
        assert_eq!(Mode::Code(Language::C).modifiers(), None);
        assert_eq!(Mode::Code(Language::C).language(), Some(Language::C));
    }
}
