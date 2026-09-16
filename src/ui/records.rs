use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, DURATIONS};
use crate::records;

/// Width of the records box, in columns.
pub const WIDTH: u16 = 48;

/// Content rows: a heading, the blank under it, and one row per duration.
/// Derived rather than written out, so adding a duration can't silently clip
/// the last row off the bottom.
const ROWS: u16 = DURATIONS.len() as u16 + 2;
/// The rows, plus a row of padding top and bottom, plus the border.
pub const HEIGHT: u16 = ROWS + 4;

/// Shown where there is no record yet — an em dash reads as "nothing here",
/// where a zero would read as a score of zero.
const NONE: &str = "—";

fn row(time: &str, best: &str, accuracy: &str, runs: &str, last: &str) -> String {
    format!("{time:>5}  {best:>8}  {accuracy:>8}  {runs:>5}   {last:<10}")
}

/// Personal bests, one row per test length.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();
    let records = app.records();

    let mut lines = vec![
        Line::from(Span::styled(
            row("time", "best", "accuracy", "runs", "set"),
            Style::default().fg(theme.dim),
        )),
        Line::from(""),
    ];

    for seconds in DURATIONS {
        let runs = records.runs(seconds);

        let line = match records.best(seconds) {
            Some(best) => Line::from(Span::styled(
                row(
                    &format!("{seconds}s"),
                    &format!("{:.0} wpm", best.wpm),
                    &format!("{:.0}%", best.accuracy),
                    &runs.to_string(),
                    &records::age(best.at),
                ),
                Style::default().fg(theme.accent),
            )),
            // Never attempted: dashes rather than zeroes.
            None => Line::from(Span::styled(
                row(&format!("{seconds}s"), NONE, NONE, "0", NONE),
                Style::default().fg(theme.dim),
            )),
        };

        lines.push(line);
    }

    let table = Paragraph::new(lines).block(super::panel(" records ", theme));

    frame.render_widget(table, area);
}
