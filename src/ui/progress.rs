use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Chart, Dataset, GraphType, Paragraph};
use ratatui::Frame;

use super::graph;
use crate::app::App;
use crate::history;
use crate::theme::Theme;

/// Rows the caption under the plot takes.
const CAPTION_HEIGHT: u16 = 1;

/// How many runs the plot shows at once.
///
/// The last fifty rather than all of them: a line squeezing a thousand runs
/// into forty columns is a texture, not a trend, and the question this answers
/// is about the shape of recent weeks.
const SHOWN: usize = 50;

/// A run count below which there is no shape to draw.
const MIN_RUNS: usize = 2;

/// Shown in place of the plot before there are two runs to draw a line between.
const TOO_FEW: &str = "not enough runs at this length yet";

/// Progress at the current test length: every run, and the trend under it.
///
/// One length rather than all three, because a 15s run and a 60s run are
/// different tests — plotting them on one line would put a step in it every
/// time you changed the setting, and call that improvement.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();
    let seconds = app.duration().as_secs();

    let [plot_area, caption_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(CAPTION_HEIGHT)]).areas(area);

    let all = app.history().at(seconds);
    // Kept as an offset rather than dropped, so the axis can still say which
    // runs these are out of everything you've typed.
    let first = all.len().saturating_sub(SHOWN);
    let runs = &all[first..];

    if runs.len() < MIN_RUNS {
        let note = Paragraph::new(TOO_FEW)
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.dim));

        frame.render_widget(note, plot_area);
        frame.render_widget(caption(seconds, theme), caption_area);
        return;
    }

    // Owned, because a `Dataset` borrows its points for as long as it lives.
    // The trend is taken over the whole history, not the window on screen: the
    // first run drawn has runs behind it, and averaging as though it didn't
    // would put a kink at the left edge that nothing in the typing caused.
    let trend = history::trend(&all, history::WINDOW);
    let speeds: Vec<(f64, f64)> = points(first, runs.iter().map(|run| run.wpm));
    let trend: Vec<(f64, f64)> = points(first, trend[first..].iter().copied());

    // The best of the whole history, marked only when it is one of the runs on
    // screen — a star over the left edge for a record set two hundred runs ago
    // would be pointing at the wrong run.
    let best: Vec<(f64, f64)> = history::best(&all)
        .filter(|index| *index >= first)
        .map(|index| vec![speeds[index - first]])
        .unwrap_or_default();

    let ceiling = speeds.iter().map(|(_, wpm)| *wpm).fold(0.0_f64, f64::max);

    let top = graph::top(ceiling);
    let (from, to) = (first as f64 + 1.0, all.len() as f64);

    let datasets = vec![
        // The runs first, so the trend is drawn over them where they cross.
        Dataset::default()
            .data(&speeds)
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme.muted)),
        Dataset::default()
            .data(&trend)
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme.accent)),
        // A dot rather than braille: a personal best is a single run, and it
        // has to be visible against the line it sits on.
        Dataset::default()
            .data(&best)
            .marker(Marker::Dot)
            .style(Style::default().fg(theme.good)),
    ];

    let chart = Chart::new(datasets)
        .x_axis(
            Axis::default()
                .bounds([from, to])
                .labels(run_ticks(from, to))
                .style(Style::default().fg(theme.dim)),
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, top])
                .labels(graph::ticks(top, ""))
                .labels_alignment(Alignment::Right)
                .style(Style::default().fg(theme.dim)),
        );

    frame.render_widget(chart, plot_area);
    frame.render_widget(caption(seconds, theme), caption_area);
}

/// Points for one line, `x` counting runs from `first` as a 1-based number.
///
/// Numbered by where a run sits in the whole history rather than in the window
/// on screen, so the axis labels mean the same thing as the run count in the
/// table above.
fn points(first: usize, values: impl Iterator<Item = f64>) -> Vec<(f64, f64)> {
    values
        .enumerate()
        .map(|(index, value)| ((first + index + 1) as f64, value))
        .collect()
}

/// Labels for the run axis: the first, the middle and the last.
///
/// Two labels when a middle one would repeat a neighbour, which is what
/// happens for the first few runs — "3 3 4" reads as a rendering fault, where
/// the ends alone read as a short history.
fn run_ticks(from: f64, to: f64) -> Vec<Line<'static>> {
    let middle = ((from + to) / 2.0).round();

    let values = if middle > from && middle < to {
        vec![from, middle, to]
    } else {
        vec![from, to]
    };

    values
        .into_iter()
        .map(|value| Line::from(format!("{value:.0}")))
        .collect()
}

/// What the two lines and the dot mean, and which test they are about.
///
/// The length is named here rather than in the panel's heading because the
/// table above it covers every length, and only the plot is about one.
fn caption(seconds: u64, theme: &Theme) -> Paragraph<'static> {
    let key = |mark: &'static str, label: String, colour| {
        [
            Span::styled(mark, Style::default().fg(colour)),
            Span::styled(format!(" {label}"), Style::default().fg(theme.dim)),
        ]
    };

    let gap = || Span::styled("   ", Style::default());

    // A separator between the keys rather than after each, so the line has no
    // trailing blanks to throw the centring off.
    let spans: Vec<Span> = [
        key("──", format!("{seconds}s runs"), theme.muted).to_vec(),
        vec![gap()],
        key("──", format!("{}-run trend", history::WINDOW), theme.accent).to_vec(),
        vec![gap()],
        key("•", "best".to_string(), theme.good).to_vec(),
    ]
    .concat();

    Paragraph::new(Line::from(spans)).alignment(Alignment::Center)
}

/// Whether there is a plot to draw at all, so the panel can leave the room out
/// rather than heading a blank space.
pub fn has_plot(app: &App) -> bool {
    app.history().at(app.duration().as_secs()).len() >= MIN_RUNS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::Run;

    /// The plain text of a set of labels.
    fn text(labels: Vec<Line<'static>>) -> Vec<String> {
        labels.iter().map(ToString::to_string).collect()
    }

    fn runs(count: usize) -> Vec<Run> {
        (0..count)
            .map(|index| Run {
                at: 1_700_000_000,
                duration: 30,
                wpm: 50.0 + index as f64,
                accuracy: 97.0,
            })
            .collect()
    }

    #[test]
    fn the_run_axis_is_labelled_at_both_ends_and_the_middle() {
        assert_eq!(text(run_ticks(1.0, 51.0)), ["1", "26", "51"]);
    }

    #[test]
    fn a_short_history_drops_the_middle_label() {
        assert_eq!(text(run_ticks(1.0, 2.0)), ["1", "2"]);
        assert_eq!(text(run_ticks(1.0, 3.0)), ["1", "2", "3"]);
    }

    #[test]
    fn a_window_is_numbered_by_where_it_sits_in_the_history() {
        // The fifty-first run is labelled 51, not 1, however many came before.
        let points = points(50, [60.0, 61.0].into_iter());

        assert_eq!(points, vec![(51.0, 60.0), (52.0, 61.0)]);
    }

    #[test]
    fn only_the_last_fifty_runs_are_plotted() {
        let all = runs(120);
        let first = all.len().saturating_sub(SHOWN);

        assert_eq!(all.len() - first, SHOWN);
        assert_eq!(first, 70);
    }

    #[test]
    fn a_short_history_is_shown_whole() {
        let all = runs(3);
        assert_eq!(all.len().saturating_sub(SHOWN), 0);
    }
}
