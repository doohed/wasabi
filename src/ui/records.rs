use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::progress;
use crate::app::{App, DURATIONS};
use crate::records;

/// Width of the records box, in columns.
pub const WIDTH: u16 = 48;

/// Content rows: the column headings, the blank under them, and one row per
/// duration. Derived rather than written out, so adding a duration can't
/// silently clip the last row off the bottom.
const ROWS: u16 = DURATIONS.len() as u16 + 2;
/// The table, plus a row of padding top and bottom, plus the heading.
pub const TABLE_HEIGHT: u16 = ROWS + 3;

/// Rows the progress plot gets: a blank to separate it from the table, the
/// plot itself, and its caption.
///
/// The plot is the same height as the one on the results screen. Everyone's
/// runs sit in a narrow band well above zero, and the axis starts at zero
/// anyway — a shorter plot squeezes that band into two rows of braille, and a
/// baseline picked to fit it instead would turn a good week into a cliff.
const PLOT_HEIGHT: u16 = 11;
/// The table with the plot under it.
pub const HEIGHT: u16 = TABLE_HEIGHT + PLOT_HEIGHT;

/// The size the records want, given the room `area` has.
///
/// The plot is dropped before the table, for the same reason the results drop
/// their graph first: the bests are the record, and the picture is a nicer way
/// of looking at what came after them. It is also dropped when there is
/// nothing to plot, so a new install gets a table rather than a table and an
/// apology.
pub fn size(area: Rect, app: &App) -> (u16, u16) {
    if area.height >= HEIGHT && progress::has_plot(app) {
        (WIDTH, HEIGHT)
    } else {
        (WIDTH, TABLE_HEIGHT)
    }
}

/// Shown where there is no record yet — an em dash reads as "nothing here",
/// where a zero would read as a score of zero.
const NONE: &str = "—";

fn row(time: &str, best: &str, accuracy: &str, runs: &str, last: &str) -> String {
    format!("{time:>5}  {best:>8}  {accuracy:>8}  {runs:>5}   {last:<10}")
}

/// Personal bests, one row per test length, and progress at the length the
/// test is set to.
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

    // The same question `size` answered, asked of the area it actually got.
    if area.height < HEIGHT || !progress::has_plot(app) {
        frame.render_widget(table, area);
        return;
    }

    let [table_area, plot_area] =
        Layout::vertical([Constraint::Length(TABLE_HEIGHT), Constraint::Fill(1)]).areas(area);

    frame.render_widget(table, table_area);

    // One row down, so the plot isn't touching the table above it.
    let [_, plot_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(plot_area);
    progress::render(frame, plot_area, app);
}
