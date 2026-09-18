use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// The best run ever recorded at one test duration.
#[derive(Debug, Clone, PartialEq)]
pub struct Best {
    pub wpm: f64,
    /// Accuracy *of that run*, not the best accuracy ever — a personal best is
    /// a single result, and splitting it across runs would flatter you.
    pub accuracy: f64,
    /// Unix seconds. Stored absolute so the file doesn't rot between sessions.
    pub at: u64,
}

/// Everything known about one test duration.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Entry {
    pub best: Option<Best>,
    /// Completed tests, including the ones that weren't personal bests.
    pub runs: usize,
}

/// Personal bests, keyed by what the test was.
///
/// A `String` rather than a number of seconds, because the length stopped
/// being the only thing that makes two tests different: see
/// [`crate::modifiers::Modifiers::key`]. A plain test's key is the bare
/// number, which is exactly what every records file written before that
/// existed already holds.
///
/// `path` is `None` for an in-memory instance, which makes [`Records::save`] a
/// no-op — that is what keeps tests off the user's real records file.
#[derive(Debug, Clone, Default)]
pub struct Records {
    entries: BTreeMap<String, Entry>,
    path: Option<PathBuf>,
}

impl Records {
    /// Read the records file, or start empty.
    ///
    /// Deliberately infallible: a missing, unreadable, or corrupt file means
    /// "no records yet". Refusing to start a typing test because a scoreboard
    /// wouldn't parse would be the wrong trade.
    pub fn load() -> Self {
        let path = records_path();
        let text = path
            .as_ref()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .unwrap_or_default();

        Self {
            entries: parse(&text),
            path,
        }
    }

    pub fn best(&self, key: &str) -> Option<&Best> {
        self.entries.get(key).and_then(|entry| entry.best.as_ref())
    }

    pub fn runs(&self, key: &str) -> usize {
        self.entries.get(key).map_or(0, |entry| entry.runs)
    }

    /// File a completed run. Returns whether it was a personal best.
    pub fn submit(&mut self, key: &str, wpm: f64, accuracy: f64) -> bool {
        let entry = self.entries.entry(key.to_string()).or_default();
        entry.runs += 1;

        let improved = entry.best.as_ref().is_none_or(|best| wpm > best.wpm);
        if improved {
            entry.best = Some(Best {
                wpm,
                accuracy,
                at: unix_now(),
            });
        }

        self.save();
        improved
    }

    /// Best-effort write. A failure here loses a score, which is not worth
    /// interrupting a typing session over.
    fn save(&self) {
        let Some(path) = &self.path else {
            return; // in-memory instance
        };

        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, format(&self.entries));
    }
}

fn records_path() -> Option<PathBuf> {
    Some(crate::storage::data_dir()?.join("records.tsv"))
}

/// Now, in unix seconds.
///
/// The history stamps its runs from here too, so both files read the clock the
/// same way and [`age`] can be pointed at either.
pub fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

/// One tab-separated line per test: `key wpm accuracy at runs`.
///
/// Hand-rolled rather than JSON because it is five numbers: a serialisation
/// dependency would outweigh the format.
fn format(entries: &BTreeMap<String, Entry>) -> String {
    let mut out = String::new();

    for (key, entry) in entries {
        let Some(best) = &entry.best else {
            continue; // nothing worth writing down
        };
        out.push_str(&format!(
            "{}\t{:.2}\t{:.2}\t{}\t{}\n",
            key, best.wpm, best.accuracy, best.at, entry.runs
        ));
    }

    out
}

/// Parse what [`format`] wrote, skipping anything that doesn't fit.
///
/// Tolerant on purpose: a half-written line from a crash costs you one score,
/// not the whole file.
fn parse(text: &str) -> BTreeMap<String, Entry> {
    let mut entries = BTreeMap::new();

    for line in text.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        let [key, wpm, accuracy, at, runs] = fields[..] else {
            continue;
        };

        // The key is whatever names the test, so nothing is parsed out of it
        // — a file from a later version naming a setting this one has never
        // heard of still reads, and simply never matches anything.
        let (Ok(wpm), Ok(accuracy), Ok(at), Ok(runs)) = (
            wpm.parse::<f64>(),
            accuracy.parse::<f64>(),
            at.parse::<u64>(),
            runs.parse::<usize>(),
        ) else {
            continue;
        };

        if key.is_empty() {
            continue;
        }

        entries.insert(
            key.to_string(),
            Entry {
                best: Some(Best { wpm, accuracy, at }),
                runs,
            },
        );
    }

    entries
}

/// How long ago a record was set, for the records table.
///
/// Relative rather than a calendar date: "3d ago" needs no timezone database
/// and is what you actually want to know about a personal best.
pub fn age(at: u64) -> String {
    let seconds = unix_now().saturating_sub(at);

    match seconds {
        0..=59 => "just now".to_string(),
        60..=3599 => format!("{}m ago", seconds / 60),
        3600..=86_399 => format!("{}h ago", seconds / 3600),
        86_400..=172_799 => "yesterday".to_string(),
        _ => format!("{}d ago", seconds / 86_400),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_first_run_is_always_a_personal_best() {
        let mut records = Records::default();
        assert!(records.submit("30", 0.0, 100.0));
        assert_eq!(records.runs("30"), 1);
    }

    #[test]
    fn only_a_faster_run_replaces_the_best() {
        let mut records = Records::default();
        records.submit("30", 60.0, 95.0);

        assert!(!records.submit("30", 50.0, 100.0));
        assert_eq!(records.best("30").unwrap().wpm, 60.0);

        assert!(records.submit("30", 70.0, 90.0));
        assert_eq!(records.best("30").unwrap().wpm, 70.0);
    }

    #[test]
    fn every_run_counts_towards_the_total() {
        let mut records = Records::default();
        for wpm in [60.0, 50.0, 70.0] {
            records.submit("30", wpm, 95.0);
        }

        assert_eq!(records.runs("30"), 3);
    }

    #[test]
    fn settings_are_scored_separately() {
        let mut records = Records::default();
        records.submit("15", 90.0, 95.0);
        records.submit("60", 40.0, 99.0);

        assert_eq!(records.best("15").unwrap().wpm, 90.0);
        assert_eq!(records.best("60").unwrap().wpm, 40.0);
        assert_eq!(records.best("30"), None);
        assert_eq!(records.runs("30"), 0);
    }

    #[test]
    fn an_unseen_duration_has_no_record() {
        let records = Records::default();
        assert_eq!(records.best("30"), None);
        assert_eq!(records.runs("30"), 0);
    }

    #[test]
    fn a_saved_file_reads_back_identically() {
        let mut entries = BTreeMap::new();
        entries.insert(
            "30".to_string(),
            Entry {
                best: Some(Best {
                    wpm: 82.5,
                    accuracy: 97.25,
                    at: 1_700_000_000,
                }),
                runs: 12,
            },
        );

        assert_eq!(parse(&format(&entries)), entries);
    }

    #[test]
    fn a_duration_never_completed_is_not_written() {
        let mut entries = BTreeMap::new();
        entries.insert("30".to_string(), Entry::default());

        assert_eq!(format(&entries), "");
    }

    #[test]
    fn junk_lines_are_skipped_not_fatal() {
        let text = "\
not a record
30\t82.50\t97.00\t1700000000\t12
15\tbroken\t97.00\t1700000000\t3
60\t70.00\t99.00\t1700000000
\t70.00\t99.00\t1700000000\t4
";
        let entries = parse(text);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries["30"].runs, 12);
    }

    #[test]
    fn an_empty_file_parses_to_no_records() {
        assert!(parse("").is_empty());
    }

    #[test]
    fn an_in_memory_records_never_touches_the_disk() {
        let mut records = Records::default();
        records.submit("30", 80.0, 99.0);

        assert!(records.path.is_none());
    }

    #[test]
    fn age_reads_in_the_largest_useful_unit() {
        let now = unix_now();
        assert_eq!(age(now), "just now");
        assert_eq!(age(now - 300), "5m ago");
        assert_eq!(age(now - 7200), "2h ago");
        assert_eq!(age(now - 90_000), "yesterday");
        assert_eq!(age(now - 86_400 * 5), "5d ago");
    }

    #[test]
    fn a_record_from_the_future_doesnt_underflow() {
        assert_eq!(age(unix_now() + 10_000), "just now");
    }
}
