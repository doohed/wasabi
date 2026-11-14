//! Which keys you keep getting wrong, across runs.
//!
//! One run's worst keys are on its results screen, but one run is a small
//! sample: a fumbled `v` in thirty seconds might be a weakness or might be a
//! sneeze. This keeps a tally over many runs, which is what the practice
//! setting draws on to decide what to drill.
//!
//! Every run's counts are added after the old ones are faded a little, so the
//! tally follows your typing as it changes: a key you have spent a week
//! practising stops being called weak, which a lifetime total would never let
//! it do.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::typing::misses::Misses;

const FILE: &str = "keys.tsv";

/// What every older count is multiplied by when a run is added.
///
/// A run twenty runs ago counts for about a third of the one just finished,
/// and one fifty runs ago for under a tenth — long enough to be more than one
/// session's luck, short enough that practising actually shows.
const DECAY: f64 = 0.95;

/// Attempts a key needs before its miss rate means anything.
///
/// Without it, a `q` met twice and missed once is the weakest key you have.
const MIN_ATTEMPTS: f64 = 10.0;

/// A tally below this, after fading, is dropped rather than carried forever.
const FORGOTTEN: f64 = 0.05;

/// One key's faded totals.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Tally {
    attempts: f64,
    misses: f64,
}

impl Tally {
    fn rate(self) -> f64 {
        self.misses / self.attempts
    }
}

/// The faded tally of every key the tests have asked for.
///
/// `path` is `None` for an in-memory instance, which makes saving a no-op —
/// that is what keeps tests off the user's real file.
#[derive(Debug, Clone, Default)]
pub struct Keys {
    tallies: BTreeMap<char, Tally>,
    path: Option<PathBuf>,
}

impl Keys {
    /// Read the file, or start empty. Infallible, like every other store: a
    /// corrupt tally means no tally yet, never a failure to start.
    pub fn load() -> Self {
        let path = keys_path();
        let text = path
            .as_ref()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .unwrap_or_default();

        Self {
            tallies: parse(&text),
            path,
        }
    }

    /// Fade what is there, then add one finished run.
    pub fn absorb(&mut self, run: &Misses) {
        for tally in self.tallies.values_mut() {
            tally.attempts *= DECAY;
            tally.misses *= DECAY;
        }

        for (key, attempts, misses) in run.counts() {
            // Whitespace is never a word's to drill, and a space would be
            // invisible in the file it is written to.
            if key.is_whitespace() {
                continue;
            }

            let tally = self.tallies.entry(key).or_default();
            tally.attempts += attempts as f64;
            tally.misses += misses as f64;
        }

        self.tallies.retain(|_, tally| tally.attempts >= FORGOTTEN);
        self.save();
    }

    /// Every key with enough behind it to judge, weakest first.
    ///
    /// Ranked by rate rather than by count, unlike one run's worst keys: over
    /// many runs the common letters pile up misses just by being common, and
    /// `e` is not the key to practise because you type it a lot.
    pub fn weakest(&self) -> Vec<char> {
        let mut weak: Vec<(char, Tally)> = self
            .tallies
            .iter()
            .filter(|(_, tally)| tally.attempts >= MIN_ATTEMPTS && tally.misses > 0.0)
            .map(|(key, tally)| (*key, *tally))
            .collect();

        // `key` last, so the order is total and can't shuffle between frames.
        weak.sort_by(|(a_key, a), (b_key, b)| b.rate().total_cmp(&a.rate()).then(a_key.cmp(b_key)));

        weak.into_iter().map(|(key, _)| key).collect()
    }

    /// Best-effort write, like the records: losing a tally is not worth
    /// interrupting a typing session over.
    fn save(&self) {
        let Some(path) = &self.path else {
            return; // in-memory instance
        };

        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, format(&self.tallies));
    }
}

fn keys_path() -> Option<PathBuf> {
    Some(crate::config::storage::data_dir()?.join(FILE))
}

/// One tab-separated line per key: `key attempts misses`.
fn format(tallies: &BTreeMap<char, Tally>) -> String {
    tallies
        .iter()
        .map(|(key, tally)| format!("{key}\t{:.3}\t{:.3}\n", tally.attempts, tally.misses))
        .collect()
}

/// Parse what [`format`] wrote, skipping anything that doesn't fit.
fn parse(text: &str) -> BTreeMap<char, Tally> {
    let mut tallies = BTreeMap::new();

    for line in text.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        let [key, attempts, misses] = fields[..] else {
            continue;
        };

        let mut chars = key.chars();
        let (Some(key), None) = (chars.next(), chars.next()) else {
            continue; // not exactly one character
        };

        let (Ok(attempts), Ok(misses)) = (attempts.parse::<f64>(), misses.parse::<f64>()) else {
            continue;
        };

        // A hand-edited line can say anything; one that claims more misses
        // than attempts, or negative ones, would rank above every real key.
        if !(attempts.is_finite() && misses.is_finite()) || misses < 0.0 || misses > attempts {
            continue;
        }

        tallies.insert(key, Tally { attempts, misses });
    }

    tallies
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A run where `key` was asked for `attempts` times and missed `misses`.
    fn run(entries: &[(char, usize, usize)]) -> Misses {
        let mut misses = Misses::default();
        for &(key, attempts, missed) in entries {
            for index in 0..attempts {
                misses.record(key, index >= missed);
            }
        }
        misses
    }

    #[test]
    fn nothing_typed_is_nothing_weak() {
        assert!(Keys::default().weakest().is_empty());
    }

    #[test]
    fn keys_are_ranked_by_rate_not_by_count() {
        let mut keys = Keys::default();
        // `e` missed more often, but out of far more attempts.
        keys.absorb(&run(&[('e', 100, 5), ('v', 20, 4)]));

        assert_eq!(keys.weakest(), vec!['v', 'e']);
    }

    #[test]
    fn a_key_met_too_rarely_is_not_judged() {
        let mut keys = Keys::default();
        keys.absorb(&run(&[('q', 2, 1), ('v', 20, 1)]));

        assert_eq!(keys.weakest(), vec!['v']);
    }

    #[test]
    fn a_key_never_missed_is_not_weak() {
        let mut keys = Keys::default();
        keys.absorb(&run(&[('e', 50, 0)]));

        assert!(keys.weakest().is_empty());
    }

    #[test]
    fn spaces_are_not_tallied() {
        let mut keys = Keys::default();
        keys.absorb(&run(&[(' ', 50, 10)]));

        assert!(keys.tallies.is_empty());
    }

    #[test]
    fn practising_a_key_brings_it_down_the_list() {
        let mut keys = Keys::default();
        keys.absorb(&run(&[('v', 20, 10), ('b', 20, 4)]));
        assert_eq!(keys.weakest()[0], 'v');

        // A long stretch of clean `v`s, and the old misses fade under them.
        for _ in 0..30 {
            keys.absorb(&run(&[('v', 20, 0), ('b', 20, 4)]));
        }

        assert_eq!(keys.weakest()[0], 'b');
    }

    #[test]
    fn a_saved_file_reads_back_as_it_was() {
        let mut keys = Keys::default();
        keys.absorb(&run(&[('v', 20, 4), ('é', 12, 3)]));

        assert_eq!(parse(&format(&keys.tallies)), keys.tallies);
    }

    #[test]
    fn junk_lines_are_skipped_not_fatal() {
        let text = "\
v\t20.000\t4.000
ab\t20\t4
x\tbroken\t1
y\t5\t9
z\t10\t-1
k\t10\t2\textra
e\t30\t3
";
        let tallies = parse(text);

        assert_eq!(tallies.keys().copied().collect::<Vec<_>>(), vec!['e', 'v']);
    }

    #[test]
    fn an_in_memory_tally_never_touches_the_disk() {
        let mut keys = Keys::default();
        keys.absorb(&run(&[('v', 20, 4)]));

        assert!(keys.path.is_none());
    }
}
