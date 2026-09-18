//! Every completed run, oldest first.
//!
//! A personal best is one number, and it only moves when you beat it: after a
//! good week of typing it says exactly what it said before. The history keeps
//! the runs that weren't bests too, which is what lets the records screen
//! answer the question a typing test is actually for — am I getting faster?
//!
//! Append-only, and never trimmed. A run is forty bytes and the file is read
//! once at startup; throwing away the early ones to save that would be
//! deleting the only part of the file that shows how far you've come.

use std::io::Write;
use std::path::PathBuf;

use crate::records::unix_now;

const FILE: &str = "history.tsv";

/// How many runs the trend line averages over.
///
/// Trailing rather than cumulative: an average taken from your very first run
/// stops moving once there are a few hundred behind it, so it answers "how
/// fast have you ever been" when the question is "how fast are you now".
pub const WINDOW: usize = 10;

/// One completed run.
///
/// The same four figures [`crate::records::Records::submit`] is given, because
/// the two are filed from the same place about the same run — a history that
/// could disagree with the records would be worse than no history.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Run {
    /// Unix seconds. Stored absolute so the file doesn't rot between sessions.
    pub at: u64,
    /// Test length in seconds — runs at different lengths aren't comparable,
    /// so nothing plots them together.
    pub duration: u64,
    pub wpm: f64,
    pub accuracy: f64,
}

/// Every run this install has finished.
///
/// `path` is `None` for an in-memory instance, which makes [`History::push`]
/// write nothing — that is what keeps tests off the user's real history.
#[derive(Debug, Clone, Default)]
pub struct History {
    runs: Vec<Run>,
    path: Option<PathBuf>,
}

impl History {
    /// Read the history file, or start empty.
    ///
    /// Infallible, like the records: a missing or corrupt file means "no
    /// history yet". An install that predates this file simply starts
    /// collecting one from its next run.
    pub fn load() -> Self {
        let path = history_path();
        let text = path
            .as_ref()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .unwrap_or_default();

        Self {
            runs: parse(&text),
            path,
        }
    }

    /// File a completed run, in memory and on disk.
    pub fn push(&mut self, duration: u64, wpm: f64, accuracy: f64) {
        let run = Run {
            at: unix_now(),
            duration,
            wpm,
            accuracy,
        };

        self.runs.push(run);
        self.append(run);
    }

    /// Every run at one test length, oldest first.
    ///
    /// Owned, because the caller plots them and a `Dataset` borrows its points
    /// for as long as it lives; a `Run` is four numbers, so this is cheaper
    /// than the lifetime it would save.
    pub fn at(&self, duration: u64) -> Vec<Run> {
        self.runs
            .iter()
            .copied()
            .filter(|run| run.duration == duration)
            .collect()
    }

    /// Add one line to the file, creating it if it isn't there.
    ///
    /// An append rather than a rewrite: the file only ever grows by a line, so
    /// there is no reason to put the whole of it back at the end of every
    /// test. Best-effort, like every other write here — losing a line of
    /// history is not worth interrupting a typing session over.
    fn append(&self, run: Run) {
        let Some(path) = &self.path else {
            return; // in-memory instance
        };

        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .and_then(|mut file| file.write_all(format(run).as_bytes()));
    }
}

/// A trailing mean of `runs`' speeds, one value per run.
///
/// The first values average over fewer runs than the rest — the alternative is
/// a line that starts nine runs in, which hides the part of the history a new
/// user actually has.
pub fn trend(runs: &[Run], window: usize) -> Vec<f64> {
    (0..runs.len())
        .map(|end| {
            let start = (end + 1).saturating_sub(window.max(1));
            let slice = &runs[start..=end];

            slice.iter().map(|run| run.wpm).sum::<f64>() / slice.len() as f64
        })
        .collect()
}

/// Where the fastest run sits in `runs`, for the marker on the graph.
///
/// The *first* of equal bests: a personal best is the run that set it, and a
/// later run matching it to two decimal places didn't.
pub fn best(runs: &[Run]) -> Option<usize> {
    // `reduce` rather than `max_by`, which keeps the *last* of equal values.
    runs.iter()
        .enumerate()
        .reduce(|best, run| if run.1.wpm > best.1.wpm { run } else { best })
        .map(|(index, _)| index)
}

fn history_path() -> Option<PathBuf> {
    Some(crate::storage::data_dir()?.join(FILE))
}

/// One tab-separated line per run: `at duration wpm accuracy`.
fn format(run: Run) -> String {
    format!(
        "{}\t{}\t{:.2}\t{:.2}\n",
        run.at, run.duration, run.wpm, run.accuracy
    )
}

/// Parse what [`format`] wrote, skipping anything that doesn't fit.
///
/// Reads the first four fields and ignores any beyond them, so a later version
/// that writes a fifth doesn't make this one's history unreadable — the same
/// trade the settings file makes by keeping keys it doesn't recognise.
/// Tolerant for the same reason as the records: a half-written line from a
/// crash costs you one run, not the file.
fn parse(text: &str) -> Vec<Run> {
    text.lines()
        .filter_map(|line| {
            let mut fields = line.split('\t');
            let at = fields.next()?.parse().ok()?;
            let duration = fields.next()?.parse().ok()?;
            let wpm = fields.next()?.parse().ok()?;
            let accuracy = fields.next()?.parse().ok()?;

            Some(Run {
                at,
                duration,
                wpm,
                accuracy,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs at one length, at the given speeds, oldest first.
    fn runs(duration: u64, speeds: &[f64]) -> Vec<Run> {
        speeds
            .iter()
            .map(|wpm| Run {
                at: 1_700_000_000,
                duration,
                wpm: *wpm,
                accuracy: 97.0,
            })
            .collect()
    }

    fn history(runs: Vec<Run>) -> History {
        History { runs, path: None }
    }

    #[test]
    fn a_pushed_run_reads_back() {
        let mut history = History::default();
        history.push(30, 82.0, 97.0);

        let runs = history.at(30);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].wpm, 82.0);
    }

    #[test]
    fn durations_are_kept_apart() {
        let mut history = History::default();
        history.push(15, 90.0, 95.0);
        history.push(30, 80.0, 96.0);
        history.push(15, 91.0, 94.0);

        assert_eq!(history.at(15).len(), 2);
        assert_eq!(history.at(30).len(), 1);
        assert!(history.at(60).is_empty());
    }

    #[test]
    fn runs_come_back_oldest_first() {
        let history = history(runs(30, &[10.0, 20.0, 30.0]));
        let speeds: Vec<f64> = history.at(30).iter().map(|run| run.wpm).collect();

        assert_eq!(speeds, vec![10.0, 20.0, 30.0]);
    }

    #[test]
    fn a_trend_averages_the_window_behind_each_run() {
        let runs = runs(30, &[10.0, 20.0, 30.0, 40.0]);

        // Window of two: each value is this run and the one before it.
        assert_eq!(trend(&runs, 2), vec![10.0, 15.0, 25.0, 35.0]);
    }

    #[test]
    fn a_trend_starts_before_the_window_is_full() {
        let runs = runs(30, &[10.0, 20.0]);

        // Averaged over what there is, rather than starting ten runs in.
        assert_eq!(trend(&runs, WINDOW), vec![10.0, 15.0]);
    }

    #[test]
    fn a_trend_of_nothing_is_nothing() {
        assert!(trend(&[], WINDOW).is_empty());
    }

    #[test]
    fn a_zero_window_still_averages_one_run() {
        let runs = runs(30, &[10.0, 20.0]);
        assert_eq!(trend(&runs, 0), vec![10.0, 20.0]);
    }

    #[test]
    fn the_best_run_is_the_fastest_one() {
        assert_eq!(best(&runs(30, &[10.0, 40.0, 30.0])), Some(1));
    }

    #[test]
    fn equal_bests_belong_to_the_run_that_set_it() {
        assert_eq!(best(&runs(30, &[40.0, 20.0, 40.0])), Some(0));
    }

    #[test]
    fn nothing_run_has_no_best() {
        assert_eq!(best(&[]), None);
    }

    #[test]
    fn a_written_line_reads_back_identically() {
        let run = Run {
            at: 1_700_000_000,
            duration: 30,
            wpm: 82.5,
            accuracy: 97.25,
        };

        assert_eq!(parse(&format(run)), vec![run]);
    }

    #[test]
    fn junk_lines_are_skipped_not_fatal() {
        let text = "\
not a run
1700000000\t30\t82.50\t97.00
1700000001\t30\tbroken\t97.00
1700000002\t30\t70.00
1700000003\t15\t91.00\t99.00
";
        let runs = parse(text);

        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].wpm, 82.5);
        assert_eq!(runs[1].duration, 15);
    }

    #[test]
    fn a_line_from_a_later_version_still_reads() {
        let runs = parse("1700000000\t30\t82.50\t97.00\t93.00\textra\n");

        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].accuracy, 97.0);
    }

    #[test]
    fn an_empty_file_parses_to_no_runs() {
        assert!(parse("").is_empty());
    }

    #[test]
    fn an_in_memory_history_never_touches_the_disk() {
        let mut history = History::default();
        history.push(30, 80.0, 99.0);

        assert!(history.path.is_none());
    }
}
