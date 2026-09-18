use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::graph;
use crate::app::App;
use crate::config::theme::Theme;
use crate::typing::misses;

/// Width of the full results dashboard, in columns.
pub const WIDTH: u16 = 64;
/// The score, the graph and the stats, plus the heading and the padding.
pub const HEIGHT: u16 = ROWS + 3;

/// Content rows of the dashboard: the score, the personal best, a gap, the
/// graph, a gap, and three rows of stats.
const ROWS: u16 = 1 + 1 + 1 + GRAPH_HEIGHT + 1 + 3;
/// Rows the graph gets, its caption included.
const GRAPH_HEIGHT: u16 = 9;

/// Width of the short results, for a terminal with no room for the graph.
pub const COMPACT_WIDTH: u16 = 44;
/// Eight rows of score, plus the heading and a row of padding each side.
pub const COMPACT_HEIGHT: u16 = 11;

/// Shown where a figure needs more of a run than there was — an em dash reads
/// as "nothing to say", where a zero would read as a score of zero.
const NONE: &str = "—";

/// The fallback has to fit wherever the dashboard doesn't, or a small terminal
/// would get no results at all. Checked at compile time, because the two pairs
/// of numbers are edited independently.
const _: () = assert!(COMPACT_WIDTH < WIDTH && COMPACT_HEIGHT < HEIGHT);

/// The size the results want, given the room `area` has.
///
/// The graph is the first thing dropped when the terminal is small, for the
/// same reason the art is: the numbers are the result, and the picture is a
/// nicer way of looking at them.
pub fn size(area: Rect) -> (u16, u16) {
    if area.width >= WIDTH && area.height >= HEIGHT {
        (WIDTH, HEIGHT)
    } else {
        (COMPACT_WIDTH, COMPACT_HEIGHT)
    }
}

/// The score, shown in place of the text once the test is over.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();

    // The title names the test length, so a score is never ambiguous about
    // which record it was competing for.
    let panel = super::panel(format!(" {}s test ", app.duration().as_secs()), theme)
        // Centred, unlike the other panels: this one's content is centred
        // too, and a heading hugging the left edge above it reads as a
        // mistake rather than a choice.
        .title_alignment(Alignment::Center);

    // The same question `size` answered, asked of the area it actually got.
    if area.width < WIDTH || area.height < HEIGHT {
        frame.render_widget(compact(app).block(panel), area);
        return;
    }

    let inner = panel.inner(area);
    frame.render_widget(panel, area);

    let [score_area, graph_area, stats_area] = Layout::vertical([
        Constraint::Length(2),                // the score and the personal best
        Constraint::Length(GRAPH_HEIGHT + 1), // the gap above it belongs to it
        Constraint::Length(4),                // a gap, and the three rows of stats
    ])
    .areas(inner);

    let score =
        Paragraph::new(vec![headline(app, theme), best(app, theme)]).alignment(Alignment::Center);

    frame.render_widget(score, score_area);

    // One row down, so the graph isn't touching the score above it.
    let [_, graph_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(graph_area);
    graph::render(frame, graph_area, app);

    let [_, stats_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(stats_area);
    frame.render_widget(stats(app, theme), stats_area);
}

/// The score itself, in the one place on the screen it can't be missed.
fn headline(app: &App, theme: &Theme) -> Line<'static> {
    Line::from(Span::styled(
        match app.wpm() {
            Some(wpm) => format!(" {wpm:.0} wpm "),
            None => " -- wpm ".to_string(),
        },
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD | Modifier::REVERSED),
    ))
}

/// Always a row, even when there's nothing to celebrate, so the box doesn't
/// change height between runs.
fn best(app: &App, theme: &Theme) -> Line<'static> {
    if !app.is_new_best() {
        return Line::from("");
    }

    Line::from(Span::styled(
        "★ personal best",
        Style::default().fg(theme.good).add_modifier(Modifier::BOLD),
    ))
}

// -- stats ----------------------------------------------------------------

/// Width the labels are padded to. Every cell is the same total width, so
/// centring the lines still leaves the columns aligned.
const LABEL_WIDTH: usize = 11;
const VALUE_WIDTH: usize = 8;

/// One `label  value` cell, labels dim so the numbers carry the row.
fn cell(label: &str, value: String, theme: &Theme) -> [Span<'static>; 2] {
    [
        Span::styled(
            format!("{label:>LABEL_WIDTH$}  "),
            Style::default().fg(theme.dim),
        ),
        Span::styled(
            format!("{value:<VALUE_WIDTH$}"),
            Style::default().fg(theme.accent),
        ),
    ]
}

/// A figure that needs more of a run than this one had.
fn or_none(value: Option<f64>, unit: &str) -> String {
    value.map_or_else(|| NONE.to_string(), |value| format!("{value:.0}{unit}"))
}

/// A character written as a key you could press.
///
/// Only a space needs the help: printed as itself it is a gap in the row with
/// a count hanging off it, which reads as a bug rather than a key.
fn key(c: char) -> String {
    if c == ' ' {
        "␣".to_string()
    } else {
        c.to_string()
    }
}

/// The keys the mistakes were made on, worst first.
///
/// The one figure here that isn't a score: accuracy says how much the run cost
/// you, and this says where to go and get it back. Always a row, like the
/// personal best, so the dashboard doesn't change height between a clean run
/// and a messy one — and `NONE` where there is nothing to name, because a
/// blank would read as a figure that failed to render.
fn worst_keys(app: &App, theme: &Theme) -> Line<'static> {
    let worst = app.misses().worst(misses::WORST);

    let value = if worst.is_empty() {
        NONE.to_string()
    } else {
        worst
            .iter()
            .map(|miss| format!("{} ×{}", key(miss.key), miss.misses))
            .collect::<Vec<_>>()
            .join("  ")
    };

    Line::from(cell("worst keys", value, theme).to_vec())
}

/// Everything the graph can't say: six figures in two rows of three, and the
/// keys that cost you under them.
fn stats(app: &App, theme: &Theme) -> Paragraph<'static> {
    let timeline = app.timeline();

    let top = [
        cell("accuracy", format!("{:.0}%", app.accuracy()), theme),
        cell("consistency", or_none(timeline.consistency(), "%"), theme),
        cell("raw wpm", or_none(app.raw_wpm(), ""), theme),
    ];

    let bottom = [
        cell(
            "time",
            format!("{:.0}s", app.elapsed().as_secs_f64()),
            theme,
        ),
        cell("peak wpm", or_none(timeline.peak(), ""), theme),
        // Keystrokes split into right and wrong, which is the detail accuracy
        // rounds away.
        cell(
            "chars",
            format!("{}/{}", app.keystrokes() - app.mistakes(), app.keystrokes()),
            theme,
        ),
    ];

    Paragraph::new(vec![
        Line::from(top.concat().to_vec()),
        Line::from(bottom.concat().to_vec()),
        worst_keys(app, theme),
    ])
    .alignment(Alignment::Center)
}

// -- small terminals ------------------------------------------------------

/// The same figures stacked in one column, for a terminal too small for the
/// dashboard. Nothing is dropped except the graph and the two-column layout.
fn compact(app: &App) -> Paragraph<'static> {
    let theme = app.theme();
    let timeline = app.timeline();
    let row = |label: &str, value: String| Line::from(cell(label, value, theme).to_vec());

    Paragraph::new(vec![
        headline(app, theme),
        best(app, theme),
        Line::from(""),
        row("accuracy", format!("{:.0}%", app.accuracy())),
        row("consistency", or_none(timeline.consistency(), "%")),
        row("time", format!("{:.0}s", app.elapsed().as_secs_f64())),
        row(
            "chars",
            format!("{}/{}", app.keystrokes() - app.mistakes(), app.keystrokes()),
        ),
        worst_keys(app, theme),
    ])
    .alignment(Alignment::Center)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(width: u16, height: u16) -> Rect {
        Rect::new(0, 0, width, height)
    }

    #[test]
    fn a_roomy_terminal_gets_the_graph() {
        assert_eq!(size(area(WIDTH, HEIGHT)), (WIDTH, HEIGHT));
        assert_eq!(size(area(200, 60)), (WIDTH, HEIGHT));
    }

    #[test]
    fn a_short_or_narrow_terminal_falls_back_to_the_figures() {
        assert_eq!(
            size(area(WIDTH, HEIGHT - 1)),
            (COMPACT_WIDTH, COMPACT_HEIGHT)
        );
        assert_eq!(
            size(area(WIDTH - 1, HEIGHT)),
            (COMPACT_WIDTH, COMPACT_HEIGHT)
        );
    }
}
