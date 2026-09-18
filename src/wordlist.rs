//! The word pool: where the test's words come from.
//!
//! The built-in pool is the 200 most common English words, which is what
//! MonkeyType's default test draws from — short and overwhelmingly ASCII, so
//! lines wrap predictably and the test measures typing rather than reading.
//!
//! It is a file rather than a constant because every other thing the user
//! might want to change is one: the art, the themes, the settings. A word list
//! is the most personal of the four — it decides what you spend your practice
//! on — and it was the only one you couldn't touch.

use std::path::{Path, PathBuf};

use rand::seq::IndexedRandom;

use crate::modifiers::Modifiers;
use crate::storage;
use crate::word::Word;

/// The pool shipped with the app, used until the user supplies their own.
const BUILT_IN: &str = include_str!("../assets/words.txt");

/// What a user's own pool is called, inside [`storage::config_dir`].
const FILE: &str = "words.txt";

/// Words generated per second of test.
///
/// Five words a second is 300 wpm — comfortably faster than anyone types — so
/// the timer is what ends a test, never the word list running dry.
pub const WORDS_PER_SECOND: usize = 5;

/// The words a test can draw on.
pub struct Wordlist {
    words: Vec<String>,
    /// Whether `words` came from the user's file rather than the built-in.
    custom: bool,
    /// Where a custom pool is read from and written to. `None` when there is
    /// nowhere to persist to, which makes editing impossible but typing fine.
    path: Option<PathBuf>,
}

impl Wordlist {
    /// The user's pool if there is a usable one, otherwise the built-in.
    ///
    /// Infallible, like the banner: an unreadable or wordless file means "no
    /// custom pool", never a failure to start. A typing test that won't open
    /// because a word list wouldn't parse would be the wrong trade.
    pub fn load() -> Self {
        let path = words_path();

        let custom = path
            .as_ref()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .map(|text| parse(&text))
            // A file with no words in it is someone who changed their mind,
            // not a pool. Treat it as absent, so the app isn't left with a
            // test it can't deal and no clue why.
            .filter(|words| !words.is_empty());

        match custom {
            Some(words) => Self {
                words,
                custom: true,
                path,
            },
            None => Self {
                words: parse(BUILT_IN),
                custom: false,
                path,
            },
        }
    }

    /// The built-in pool with nowhere to save to, for tests.
    #[cfg(test)]
    pub fn detached() -> Self {
        Self {
            words: parse(BUILT_IN),
            custom: false,
            path: None,
        }
    }

    pub fn is_custom(&self) -> bool {
        self.custom
    }

    /// How many words the pool holds.
    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// A test's worth of words for a run of `seconds`, modifiers applied.
    ///
    /// Drawn with repetition on purpose: sampling without it would bias the
    /// tail of a long test towards the rare, awkward words once the common
    /// ones are used up — and would run dry entirely on a short pool.
    pub fn deal(&self, seconds: u64, modifiers: Modifiers) -> Vec<Word> {
        let mut rng = rand::rng();
        let count = seconds as usize * WORDS_PER_SECOND;

        let mut words: Vec<String> = (0..count)
            .map(|_| {
                self.words
                    .choose(&mut rng)
                    .expect("a wordlist is never empty")
                    .clone()
            })
            .collect();

        modifiers.apply(&mut words, &mut rng);

        words.iter().map(|word| Word::new(word)).collect()
    }

    /// Make sure there is a file to edit, seeded with the current pool.
    ///
    /// Seeded rather than empty so `$EDITOR` opens something to modify, and so
    /// the built-in is there as a worked example of the expected shape.
    pub fn seed(&self) -> std::io::Result<&Path> {
        let path = self
            .path
            .as_deref()
            .ok_or_else(|| std::io::Error::other("no config directory to save a word list in"))?;

        if !path.exists() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, seed_text(&self.words))?;
        }

        Ok(path)
    }

    /// Delete the user's pool and go back to the built-in.
    ///
    /// Sets the built-in directly rather than reloading, for the same reason
    /// the banner does: with the file gone that is what a reload would return,
    /// and reloading a pool with no path would read the real file.
    pub fn reset(&mut self) -> std::io::Result<()> {
        if let Some(path) = &self.path {
            match std::fs::remove_file(path) {
                Ok(()) => {}
                // Already gone is the state we wanted anyway.
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }

        self.words = parse(BUILT_IN);
        self.custom = false;
        Ok(())
    }
}

fn words_path() -> Option<PathBuf> {
    Some(storage::config_dir()?.join(FILE))
}

/// Every word in `text`, in the order they appear.
///
/// Split on whitespace rather than on lines, so one word per line and a
/// paragraph of prose are both valid files — the app measures whatever you
/// save. A line starting with `#` is a comment, which is what lets the seeded
/// file explain itself; it costs you words beginning with `#`, and a typing
/// test is a fair place to spend those.
fn parse(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .flat_map(str::split_whitespace)
        .map(str::to_string)
        .collect()
}

/// The file a first `e` press creates: the current pool, and a note saying
/// what it is.
fn seed_text(words: &[String]) -> String {
    let mut out = String::from(
        "# Your word list. One word per line, or several to a line — the test\n\
         # draws from all of them at random, with repetition.\n\
         #\n\
         # Lines starting with # are ignored. Delete the lot and put your own\n\
         # words here; press r in the words screen to reload, or x to go back\n\
         # to the built-in list.\n\n",
    );

    for word in words {
        out.push_str(word);
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A pool over `words` with nowhere to save to, so tests never touch the
    /// user's real word list.
    fn wordlist(words: &[&str]) -> Wordlist {
        Wordlist {
            words: words.iter().map(|word| word.to_string()).collect(),
            custom: true,
            path: None,
        }
    }

    #[test]
    fn the_built_in_pool_is_the_two_hundred_common_words() {
        let wordlist = Wordlist::detached();

        assert_eq!(wordlist.len(), 200);
        assert!(!wordlist.is_custom());
        assert!(wordlist.words.contains(&"the".to_string()));
    }

    #[test]
    fn a_test_is_five_words_a_second() {
        let words = wordlist(&["cat"]).deal(30, Modifiers::default());
        assert_eq!(words.len(), 30 * WORDS_PER_SECOND);
    }

    #[test]
    fn a_one_word_pool_still_deals_a_full_test() {
        // Drawn with repetition, so a pool shorter than the test is fine.
        let words = wordlist(&["cat"]).deal(15, Modifiers::default());

        assert_eq!(words.len(), 15 * WORDS_PER_SECOND);
        assert!(words.iter().all(|word| word.target == "cat"));
    }

    #[test]
    fn one_word_per_line_parses() {
        assert_eq!(parse("cat\ndog\n"), vec!["cat", "dog"]);
    }

    #[test]
    fn several_words_to_a_line_parse_too() {
        assert_eq!(parse("cat dog\n  bird  "), vec!["cat", "dog", "bird"]);
    }

    #[test]
    fn comments_and_blank_lines_are_not_words() {
        let words = parse("# a note\ncat\n\n   # indented note\ndog\n");
        assert_eq!(words, vec!["cat", "dog"]);
    }

    #[test]
    fn a_file_of_nothing_but_comments_has_no_words() {
        assert!(parse("# nothing here\n\n").is_empty());
    }

    #[test]
    fn a_seeded_file_reads_back_as_the_pool_it_came_from() {
        let words = vec!["cat".to_string(), "dog".to_string()];
        assert_eq!(parse(&seed_text(&words)), words);
    }

    #[test]
    fn resetting_a_detached_wordlist_is_harmless() {
        // No path, so there is no file to remove and nothing read from the
        // user's real one.
        let mut wordlist = wordlist(&["cat"]);
        assert!(wordlist.is_custom());

        assert!(wordlist.reset().is_ok());
        assert!(!wordlist.is_custom());
        assert_eq!(wordlist.len(), 200);
    }

    #[test]
    fn a_wordlist_with_nowhere_to_save_cant_be_seeded() {
        assert!(wordlist(&["cat"]).seed().is_err());
    }
}
