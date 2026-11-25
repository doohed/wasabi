use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::{kit, progress};
use crate::app::{App, LENGTHS};
use crate::config::theme::Theme;
use crate::scores::records;

/// Width of the records box, in columns.
pub const WIDTH: u16 = 54;
/// Columns the rows span, inside the margins.
const ROW: usize = kit::content(WIDTH);

/// Content rows: the column headings, the rule under them, and one row per
/// length. Derived rather than written out, so adding a length can't
/// silently clip the last row off the bottom.
const ROWS: u16 = LENGTHS.len() as u16 + 2;
/// The table, plus a row of padding top and bottom, plus the heading.
pub const TABLE_HEIGHT: u16 = ROWS + 3;

/// Rows the progress plot gets: a heading to separate it from the table, the
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

/// The table's columns, right-aligned so the numbers line up.
const COLUMNS: [usize; 5] = [5, 9, 10, 6, 4];

/// One row of the table: a cell per column, each in its own style.
fn row(cells: [(String, Style); 5], highlighted: bool, theme: &Theme) -> Line<'static> {
    let mut spans = kit::bar(highlighted, theme);
    for (index, ((text, style), width)) in cells.into_iter().zip(COLUMNS).enumerate() {
        // The last column is a phrase, not a number: it reads from the left.
        let cell = if index == COLUMNS.len() - 1 {
            format!("    {text}")
        } else {
            format!("{text:>width$}")
        };
        spans.push(Span::styled(cell, style));
    }
    Line::from(spans)
}

/// Personal bests, one row per test length, and progress at the length the
/// test is set to.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();
    let records = app.records();
    let dim = Style::default().fg(theme.dim);

    let mut lines = vec![
        row(
            ["test", "best", "accuracy", "runs", "set"].map(|h| (h.to_string(), dim)),
            false,
            theme,
        ),
        Line::from(vec![
            Span::raw(" ".repeat(kit::MARGIN)),
            Span::styled("─".repeat(ROW), dim),
        ]),
    ];

    // Every length, at the setting in force: the table is about how long you
    // type for, and the panel's heading says what kind of test it was.
    let mode = app.mode();

    for length in LENGTHS {
        let key = mode.key(length);
        // The bar marks the length the test is set to — the one the plot
        // below is about.
        let highlighted = length == app.length();
        let name = (length.short(), kit::name(highlighted, theme));
        let runs = records.runs(&key);

        let cells = match records.best(&key) {
            Some(best) => [
                name,
                (
                    format!("{:.0} wpm", best.wpm),
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                (
                    format!("{:.0}%", best.accuracy),
                    Style::default().fg(theme.text),
                ),
                (runs.to_string(), Style::default().fg(theme.muted)),
                (records::age(best.at), dim),
            ],
            // Never attempted: dashes rather than zeroes.
            None => [
                name,
                (NONE.to_string(), dim),
                (NONE.to_string(), dim),
                ("0".to_string(), dim),
                (NONE.to_string(), dim),
            ],
        };

        lines.push(row(cells, highlighted, theme));
    }

    let title = match mode.label() {
        Some(label) => format!(" records · {label} "),
        None => " records ".to_string(),
    };

    let table = Paragraph::new(lines).block(super::panel(title, theme));

    // The same question `size` answered, asked of the area it actually got.
    if area.height < HEIGHT || !progress::has_plot(app) {
        frame.render_widget(table, area);
        return;
    }

    let [table_area, plot_area] =
        Layout::vertical([Constraint::Length(TABLE_HEIGHT), Constraint::Fill(1)]).areas(area);

    frame.render_widget(table, table_area);

    // A heading between the two, so the plot isn't touching the table.
    let [heading_area, plot_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(plot_area);
    frame.render_widget(
        Paragraph::new(kit::heading(
            &format!("progress · {}", app.length().short()),
            ROW,
            theme,
        )),
        heading_area,
    );
    progress::render(frame, plot_area, app);
}
