use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Chart, Dataset, GraphType, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::config::theme::Theme;
use crate::typing::timeline::Sample;

/// Rows the caption under the plot takes.
const CAPTION_HEIGHT: u16 = 1;

/// The gap between gridlines on the speed axis, in wpm.
///
/// Rounding the top of the axis up to a multiple of this keeps the labels
/// round numbers, and stops two runs at similar speeds from getting charts
/// that look nothing alike.
const GRIDLINE: f64 = 20.0;

/// Where the speed axis ends, given the highest point it has to fit.
///
/// Shared with the progress graph rather than written twice, so the two charts
/// in the app can't end up rounding their speeds differently.
pub(super) fn top(ceiling: f64) -> f64 {
    (ceiling / GRIDLINE).ceil().max(1.0) * GRIDLINE
}

/// Shown in place of the plot when the run was too short to read even once.
const TOO_SHORT: &str = "not enough of a run to plot";

/// The pace of the run: what the speed was doing, second by second.
///
/// Two lines on one scale rather than the usual two axes. `raw` is the speed
/// of each second on its own and `wpm` is the running score, so the gap
/// between them is the run's own history catching up with it — which only
/// reads as that if both are measured in the same units.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();
    let timeline = app.timeline();

    let [plot_area, caption_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(CAPTION_HEIGHT)]).areas(area);

    let (Some(span), Some(ceiling)) = (timeline.span(), timeline.ceiling()) else {
        // A run under a second. The stats below still have something to say,
        // so this replaces the plot rather than the whole panel.
        let note = Paragraph::new(TOO_SHORT)
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.dim));

        frame.render_widget(note, plot_area);
        return;
    };

    // Owned, because a `Dataset` borrows its points for as long as it lives.
    let wpm = line(timeline.samples(), |sample| sample.wpm);
    let raw = line(timeline.samples(), |sample| sample.raw);

    // Marked on the raw line rather than on the score: a mistake belongs to
    // the second you made it in, and that is the line that shows that second.
    let errors: Vec<(f64, f64)> = timeline
        .samples()
        .iter()
        .filter(|sample| sample.mistakes > 0)
        .map(|sample| (sample.at, sample.raw))
        .collect();

    let top = top(ceiling);

    // Whole seconds, so the labels of a 15s test read 0s / 8s / 15s. The
    // readings land a few milliseconds late — a frame can't fire on the
    // instant — and labelling that honestly would put "7s" in the middle of a
    // fifteen second test.
    let span = span.round().max(1.0);

    let datasets = vec![
        // Raw first, so the score is drawn over it where the two cross.
        Dataset::default()
            .data(&raw)
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            // Muted rather than dim, which is the axis's colour: the second
            // line has to be quieter than the score and louder than the frame.
            .style(Style::default().fg(theme.muted)),
        Dataset::default()
            .data(&wpm)
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme.accent)),
        // A dot rather than braille: an error is a single moment, and it has
        // to be visible against the line it sits on.
        Dataset::default()
            .data(&errors)
            .marker(Marker::Dot)
            .style(Style::default().fg(theme.error)),
    ];

    let chart = Chart::new(datasets)
        .x_axis(
            Axis::default()
                .bounds([0.0, span])
                .labels(ticks(span, "s"))
                .style(Style::default().fg(theme.dim)),
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, top])
                .labels(ticks(top, ""))
                .labels_alignment(Alignment::Right)
                .style(Style::default().fg(theme.dim)),
        );

    frame.render_widget(chart, plot_area);
    frame.render_widget(caption(theme), caption_area);
}

/// Three labels for an axis running from zero to `end`, `unit` appended.
///
/// Rounded here rather than left to the formatter, which breaks a tie towards
/// the even number: half of five is three, not two.
pub(super) fn ticks(end: f64, unit: &str) -> Vec<Line<'static>> {
    [0.0, end / 2.0, end]
        .map(|value| Line::from(format!("{}{unit}", value.round())))
        .to_vec()
}

/// The points of one line, anchored at the left edge of the plot.
///
/// A reading describes the second *ending* at its timestamp, so the first one
/// sits a whole second in and leaves the line starting in mid-air, clear of
/// the axis. Repeating it at zero draws it across the second it actually
/// covers — a flat start rather than a gap, and without inventing a climb the
/// run didn't have.
fn line(samples: &[Sample], value: fn(&Sample) -> f64) -> Vec<(f64, f64)> {
    let anchor = samples.first().map(|first| (0.0, value(first)));

    anchor
        .into_iter()
        .chain(samples.iter().map(|sample| (sample.at, value(sample))))
        .collect()
}

/// What the two lines and the dots mean.
///
/// A caption rather than the chart's own legend: a legend is a box floating
/// over the plot, and this one would sit exactly where a fast run's line goes.
fn caption(theme: &Theme) -> Paragraph<'static> {
    let key = |mark: &'static str, label: &'static str, colour| {
        [
            Span::styled(mark, Style::default().fg(colour)),
            Span::styled(format!(" {label}"), Style::default().fg(theme.dim)),
        ]
    };

    let gap = || Span::styled("   ", Style::default());

    // A separator between the keys rather than after each, so the line has no
    // trailing blanks to throw the centring off.
    let spans: Vec<Span> = [
        key("──", "wpm", theme.accent).to_vec(),
        vec![gap()],
        key("──", "raw", theme.muted).to_vec(),
        vec![gap()],
        key("•", "error", theme.error).to_vec(),
    ]
    .concat();

    Paragraph::new(Line::from(spans)).alignment(Alignment::Center)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The plain text of a set of labels.
    fn text(labels: Vec<Line<'static>>) -> Vec<String> {
        labels.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn an_axis_is_labelled_at_both_ends_and_the_middle() {
        assert_eq!(text(ticks(30.0, "s")), ["0s", "15s", "30s"]);
        assert_eq!(text(ticks(120.0, "")), ["0", "60", "120"]);
    }

    #[test]
    fn a_ragged_end_is_labelled_with_a_round_number() {
        // The last reading lands a fraction past the end of the clock, and
        // "30s" is what the user set the test to.
        assert_eq!(text(ticks(30.07, "s")), ["0s", "15s", "30s"]);
    }

    #[test]
    fn an_odd_length_gets_a_whole_second_in_the_middle() {
        // Half of fifteen is 7.5, and the formatter alone would call that 8
        // here and 2 for a span of 5 — so it isn't left to the formatter.
        assert_eq!(text(ticks(15.0, "s")), ["0s", "8s", "15s"]);
        assert_eq!(text(ticks(5.0, "s")), ["0s", "3s", "5s"]);
    }

    /// Readings a second apart, each `raw` faster than the last.
    fn samples(count: usize) -> Vec<Sample> {
        (1..=count)
            .map(|second| Sample {
                at: second as f64,
                wpm: 50.0,
                raw: second as f64 * 10.0,
                mistakes: 0,
            })
            .collect()
    }

    #[test]
    fn a_line_starts_at_the_axis_rather_than_a_second_in() {
        let points = line(&samples(3), |sample| sample.raw);

        assert_eq!(points.len(), 4);
        // The first reading, drawn across the second it describes.
        assert_eq!(points[0], (0.0, 10.0));
        assert_eq!(points[1], (1.0, 10.0));
        assert_eq!(points[3], (3.0, 30.0));
    }

    #[test]
    fn a_line_with_no_readings_has_nothing_to_anchor() {
        assert!(line(&[], |sample| sample.raw).is_empty());
    }
}
