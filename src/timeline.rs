//! The shape of a run: what the score was doing while it happened.
//!
//! A single final number says how fast you typed; it says nothing about
//! whether you held that speed or bought it with one lucky burst. The
//! timeline keeps a reading a second so the results can draw that.

/// The divisor in the standard WPM definition: a "word" is five characters,
/// regardless of where the spaces fall.
const CHARS_PER_WORD: f64 = 5.0;

/// Words per minute: `chars` characters typed over `seconds` seconds.
///
/// The one definition of the score in the app — the header, the records and
/// every point on the graph come through here, so they can't disagree.
/// Zero for a zero-length span, which is the divide-by-zero guard.
pub fn wpm(chars: usize, seconds: f64) -> f64 {
    if seconds <= 0.0 {
        return 0.0;
    }

    (chars as f64 / CHARS_PER_WORD) / (seconds / 60.0)
}

/// One reading, covering roughly a second of a run.
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    /// Seconds into the test the reading was taken.
    pub at: f64,
    /// Net words per minute for the run *so far* — the figure the header was
    /// showing at that moment.
    pub wpm: f64,
    /// Words per minute over this reading's slice alone, mistakes included.
    ///
    /// Jagged where [`Sample::wpm`] is smooth, because it isn't averaged over
    /// the whole run — which is exactly what makes the shape worth drawing.
    pub raw: f64,
    /// Mistakes made during this slice.
    pub mistakes: usize,
}

/// Every reading taken during one run, oldest first.
///
/// Fed the run's running totals and left to decide when a reading is due, so
/// the caller doesn't have to keep a clock of its own.
#[derive(Debug, Clone, Default)]
pub struct Timeline {
    samples: Vec<Sample>,
    /// The totals as of the last reading, which is what makes each slice a
    /// difference rather than a total.
    at: f64,
    keystrokes: usize,
    mistakes: usize,
}

/// How much time has to pass before another reading is taken, in seconds.
///
/// A second is short enough to show the shape of a 15s run and long enough
/// that dividing by it gives an honest speed.
const INTERVAL: f64 = 1.0;

impl Timeline {
    /// Throw the run away. Called on restart, like every other run statistic.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Offer the run's totals; a reading is taken only once a whole second
    /// has passed since the last one.
    ///
    /// Called every frame rather than every second — the event loop has no
    /// second hand, and a short slice divides into a wild speed, so the
    /// decision lives here where the last reading's time is known.
    pub fn tick(&mut self, at: f64, correct: usize, keystrokes: usize, mistakes: usize) {
        let span = at - self.at;
        if span < INTERVAL {
            return;
        }

        self.samples.push(Sample {
            at,
            wpm: wpm(correct, at),
            // Against the true span, not against `INTERVAL`: a frame can land
            // late, and pretending the slice was a second would inflate it.
            raw: wpm(keystrokes.saturating_sub(self.keystrokes), span),
            mistakes: mistakes.saturating_sub(self.mistakes),
        });

        self.at = at;
        self.keystrokes = keystrokes;
        self.mistakes = mistakes;
    }

    /// The readings, for the graph.
    pub fn samples(&self) -> &[Sample] {
        &self.samples
    }

    /// The fastest second of the run, in raw wpm.
    ///
    /// `None` for a run too short to have been read even once.
    pub fn peak(&self) -> Option<f64> {
        self.samples
            .iter()
            .map(|sample| sample.raw)
            .max_by(f64::total_cmp)
    }

    /// The highest point anything drawn reaches, for the graph's y axis.
    pub fn ceiling(&self) -> Option<f64> {
        self.samples
            .iter()
            .map(|sample| sample.raw.max(sample.wpm))
            .max_by(f64::total_cmp)
    }

    /// Seconds covered by the readings — where the graph's x axis ends.
    pub fn span(&self) -> Option<f64> {
        self.samples.last().map(|sample| sample.at)
    }

    /// How even the pace was, as a percentage. 100% is a metronome.
    ///
    /// The coefficient of variation of the per-second speeds, turned the right
    /// way up: spread relative to the mean, so a steady 40 wpm scores as well
    /// as a steady 100. `None` until there are two readings to vary between,
    /// because one number has no spread and calling that perfect would be a
    /// free 100% for anyone who stops at a second.
    pub fn consistency(&self) -> Option<f64> {
        if self.samples.len() < 2 {
            return None;
        }

        let count = self.samples.len() as f64;
        let mean = self.samples.iter().map(|sample| sample.raw).sum::<f64>() / count;

        // Typed nothing at all: no spread, but no consistency either.
        if mean <= 0.0 {
            return Some(0.0);
        }

        let variance = self
            .samples
            .iter()
            .map(|sample| (sample.raw - mean).powi(2))
            .sum::<f64>()
            / count;

        // Clamped because a single burst against a mostly idle run can put the
        // deviation above the mean, and "-40% consistent" means nothing.
        Some((1.0 - variance.sqrt() / mean).clamp(0.0, 1.0) * 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A timeline read once a second, fed `chars` correct characters and
    /// `mistakes` mistakes in every one of `seconds` seconds.
    fn steady(seconds: usize, chars: usize, mistakes: usize) -> Timeline {
        let mut timeline = Timeline::default();

        for second in 1..=seconds {
            timeline.tick(
                second as f64,
                chars * second,
                chars * second,
                mistakes * second,
            );
        }

        timeline
    }

    #[test]
    fn five_characters_a_second_is_sixty_wpm() {
        assert_eq!(wpm(5, 1.0), 60.0);
        assert_eq!(wpm(50, 60.0), 10.0);
    }

    #[test]
    fn a_zero_length_span_scores_nothing_rather_than_dividing_by_zero() {
        assert_eq!(wpm(10, 0.0), 0.0);
    }

    #[test]
    fn a_reading_is_taken_once_a_second() {
        let mut timeline = Timeline::default();

        // Ten frames inside the first second: still nothing to plot.
        for frame in 1..=9 {
            timeline.tick(f64::from(frame) / 10.0, 5, 5, 0);
        }
        assert!(timeline.samples().is_empty());

        timeline.tick(1.0, 5, 5, 0);
        assert_eq!(timeline.samples().len(), 1);
    }

    #[test]
    fn each_reading_covers_only_its_own_slice() {
        let timeline = steady(3, 10, 1);
        let samples = timeline.samples();

        assert_eq!(samples.len(), 3);
        // 10 characters in each second, every second.
        for sample in samples {
            assert_eq!(sample.raw, 120.0);
            assert_eq!(sample.mistakes, 1);
        }
    }

    #[test]
    fn the_running_score_is_cumulative_where_the_raw_one_is_not() {
        let mut timeline = Timeline::default();
        timeline.tick(1.0, 10, 10, 0); // a fast first second
        timeline.tick(2.0, 10, 10, 0); // and a second spent thinking

        let samples = timeline.samples();
        assert_eq!(
            samples[1].raw, 0.0,
            "nothing was typed in the second second"
        );
        // 10 correct characters over two seconds: half the first reading.
        assert_eq!(samples[1].wpm, samples[0].wpm / 2.0);
    }

    #[test]
    fn a_late_frame_is_scored_over_the_time_it_actually_took() {
        let mut timeline = Timeline::default();
        // One two-second slice, not a second's worth of credit for two.
        timeline.tick(2.0, 10, 10, 0);

        assert_eq!(timeline.samples()[0].raw, 60.0);
    }

    #[test]
    fn an_even_pace_is_perfectly_consistent() {
        assert_eq!(steady(5, 10, 0).consistency(), Some(100.0));
    }

    #[test]
    fn an_uneven_pace_scores_lower_than_an_even_one() {
        // Running totals, so the slices are 20, 20, 5, 30 and 30 characters.
        let mut jagged = Timeline::default();
        for (second, typed) in [(1, 20), (2, 40), (3, 45), (4, 75), (5, 105)] {
            jagged.tick(f64::from(second), typed, typed, 0);
        }

        let jagged = jagged.consistency().expect("five readings");
        assert!(jagged < 100.0, "got {jagged}");
        assert!(jagged > 0.0, "got {jagged}");
    }

    #[test]
    fn one_reading_is_not_enough_to_call_a_pace_consistent() {
        let mut timeline = Timeline::default();
        timeline.tick(1.0, 10, 10, 0);

        assert_eq!(timeline.consistency(), None);
    }

    #[test]
    fn a_run_nobody_typed_is_not_consistent() {
        assert_eq!(steady(4, 0, 0).consistency(), Some(0.0));
    }

    #[test]
    fn the_peak_is_the_fastest_single_second() {
        // Running totals again: 5 characters, then 15, then 5.
        let mut timeline = Timeline::default();
        for (second, typed) in [(1, 5), (2, 20), (3, 25)] {
            timeline.tick(f64::from(second), typed, typed, 0);
        }

        assert_eq!(timeline.peak(), Some(180.0));
        assert_eq!(timeline.span(), Some(3.0));
    }

    #[test]
    fn an_unread_run_has_nothing_to_plot() {
        let timeline = Timeline::default();

        assert!(timeline.samples().is_empty());
        assert_eq!(timeline.peak(), None);
        assert_eq!(timeline.ceiling(), None);
        assert_eq!(timeline.span(), None);
    }

    #[test]
    fn the_ceiling_clears_both_lines() {
        let mut timeline = Timeline::default();
        timeline.tick(1.0, 10, 10, 0); // raw and wpm both 120
        timeline.tick(2.0, 10, 10, 0); // wpm falls to 60, raw to 0

        assert_eq!(timeline.ceiling(), Some(120.0));
    }

    #[test]
    fn clearing_forgets_the_run_and_its_totals() {
        let mut timeline = steady(3, 10, 1);
        timeline.clear();
        assert!(timeline.samples().is_empty());

        // The next run starts from zero, not from the last one's totals.
        timeline.tick(1.0, 10, 10, 0);
        assert_eq!(timeline.samples()[0].raw, 120.0);
    }
}
