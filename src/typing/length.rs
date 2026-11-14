//! How long a test runs: for a time, or for a number of words.
//!
//! The two are different questions. A timed test asks how much you can type
//! before the clock stops; a word test asks how long a fixed amount of typing
//! takes you, which is closer to what typing is for — you never stop writing
//! an email because thirty seconds are up. One enum, so a test is always
//! exactly one of them and can never be both or neither.

use std::time::Duration;

use super::wordlist::WORDS_PER_SECOND;

/// The length of one test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Length {
    /// Runs until the clock stops, this many seconds after the first key.
    Time(u64),
    /// Runs until this many words are typed, however long that takes.
    Words(usize),
}

impl Length {
    /// What it is called in a file: the settings, and every record key.
    ///
    /// A timed test is the bare number, which is what every records file and
    /// settings file written before word tests existed already holds — so
    /// adding them orphaned nothing. A word test is prefixed, so `w25` can
    /// never be mistaken for twenty-five seconds by a build that predates it:
    /// that build fails to parse it and falls back to its default.
    pub fn slug(self) -> String {
        match self {
            Length::Time(seconds) => seconds.to_string(),
            Length::Words(count) => format!("w{count}"),
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug.strip_prefix('w') {
            Some(count) => count.parse().ok().map(Length::Words),
            None => slug.parse().ok().map(Length::Time),
        }
    }

    /// What the menu calls it.
    pub fn label(self) -> String {
        match self {
            Length::Time(seconds) => format!("{seconds} seconds"),
            Length::Words(count) => format!("{count} words"),
        }
    }

    /// The narrowest name it has, for table columns and captions.
    pub fn short(self) -> String {
        match self {
            Length::Time(seconds) => format!("{seconds}s"),
            Length::Words(count) => format!("{count}w"),
        }
    }

    /// The heading of the results panel.
    pub fn title(self) -> String {
        match self {
            Length::Time(seconds) => format!("{seconds}s test"),
            Length::Words(count) => format!("{count}-word test"),
        }
    }

    /// How many words to deal.
    ///
    /// A timed test is dealt more than anyone can type — see
    /// [`WORDS_PER_SECOND`] — so the clock ends it. A word test is dealt
    /// exactly its length, so running out of words *is* the end.
    pub fn count(self) -> usize {
        match self {
            Length::Time(seconds) => seconds as usize * WORDS_PER_SECOND,
            Length::Words(count) => count,
        }
    }

    /// How long the clock gives you, or `None` when there is no clock.
    pub fn limit(self) -> Option<Duration> {
        match self {
            Length::Time(seconds) => Some(Duration::from_secs(seconds)),
            Length::Words(_) => None,
        }
    }
}

/// A bare number is a number of seconds, which is what every record key was
/// before word tests existed — and what keeps `mode.key(30)` reading as it
/// always did.
impl From<u64> for Length {
    fn from(seconds: u64) -> Self {
        Length::Time(seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_length_survives_a_trip_through_its_slug() {
        for length in [Length::Time(30), Length::Words(25), Length::Words(100)] {
            assert_eq!(Length::from_slug(&length.slug()), Some(length));
        }
    }

    #[test]
    fn a_timed_test_keeps_the_bare_number() {
        // What every settings and records file written before this holds.
        assert_eq!(Length::Time(30).slug(), "30");
        assert_eq!(Length::from_slug("15"), Some(Length::Time(15)));
    }

    #[test]
    fn junk_is_not_a_length() {
        assert_eq!(Length::from_slug("ages"), None);
        assert_eq!(Length::from_slug("w"), None);
        assert_eq!(Length::from_slug("wx"), None);
        assert_eq!(Length::from_slug(""), None);
    }

    #[test]
    fn a_word_test_deals_exactly_its_length_and_has_no_clock() {
        assert_eq!(Length::Words(25).count(), 25);
        assert_eq!(Length::Words(25).limit(), None);
    }

    #[test]
    fn a_timed_test_deals_more_than_anyone_can_type() {
        assert_eq!(Length::Time(30).count(), 30 * WORDS_PER_SECOND);
        assert_eq!(Length::Time(30).limit(), Some(Duration::from_secs(30)));
    }
}
