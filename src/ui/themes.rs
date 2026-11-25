use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::kit;
use crate::app::App;
use crate::config::storage;
use crate::config::theme::Theme;

/// Width of the picker, in columns.
///
/// Set by the widest row: the name, the palette, and the marker beside a
/// theme of the user's own — with room for the longest reload message.
pub const WIDTH: u16 = 52;

/// Columns one colour of the palette gets.
///
/// Wide enough to judge a hue by, and narrow enough that nine of them sit
/// beside the name rather than pushing it off the row.
const SWATCH_WIDTH: usize = 2;

/// Most theme rows shown at once.
///
/// Someone with thirty themes gets a scrolling list rather than a box taller
/// than their terminal.
const MAX_ROWS: usize = 8;

/// Columns the rows span, inside the margins.
const ROW: usize = kit::content(WIDTH);

/// Rows that aren't themes: the list's heading, then the keys' heading, two
/// keys, the file, a blank and the status line.
const CHROME_ROWS: u16 = 7;

/// How many theme rows are on screen.
fn visible(app: &App) -> usize {
    app.themes().all().len().min(MAX_ROWS)
}

/// Height the picker needs, which depends on how many themes exist.
pub fn height(app: &App) -> u16 {
    // The rows, the rest, then the title and a row of padding each side.
    visible(app) as u16 + CHROME_ROWS + 3
}

/// First row to draw, so the selection is always on screen.
///
/// Scrolls only as far as it must, so short lists never move at all.
fn offset(selected: usize, total: usize, visible: usize) -> usize {
    if selected < visible {
        0
    } else {
        (selected + 1 - visible).min(total.saturating_sub(visible))
    }
}

/// The theme's whole palette, as a row of chips.
///
/// In the order a theme file lists them, so the row and the file you edit read
/// the same way round. Spaced rather than butted together: `text`, `muted` and
/// `dim` are three neutrals in a row, and with no gap between them a palette's
/// near-identical colours read as one wider block instead of two.
fn swatch(theme: &Theme) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    for colour in theme.palette() {
        if !spans.is_empty() {
            spans.push(Span::raw(" "));
        }

        spans.push(Span::styled(
            "█".repeat(SWATCH_WIDTH),
            Style::default().fg(colour),
        ));
    }

    spans
}

/// The theme picker.
///
/// The rest of the interface is already drawn in the highlighted theme — the
/// preview is the app itself, not a sample inside this box.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let current = app.theme();
    let all = app.themes().all();
    let selected = app.theme_index();
    let visible = visible(app);
    let first = offset(selected, all.len(), visible);

    let mut lines = vec![kit::heading(
        &format!("themes · {} of {}", selected + 1, all.len()),
        ROW,
        current,
    )];

    lines.extend(
        all.iter()
            .enumerate()
            .skip(first)
            .take(visible)
            .map(|(index, theme)| {
                let highlighted = index == selected;

                let mut left = kit::bar(highlighted, current);
                left.push(Span::styled(
                    format!("{:<12}", theme.name),
                    kit::name(highlighted, current),
                ));
                // Marks a theme as one of the user's own, so it's obvious
                // which rows come from their file.
                if theme.custom {
                    left.push(Span::styled(
                        "yours",
                        Style::default()
                            .fg(current.accent)
                            .add_modifier(Modifier::ITALIC),
                    ));
                }

                kit::spread(left, swatch(theme), ROW)
            }),
    );

    // Shown so that someone with no $EDITOR still knows which file to open.
    let file = match app.themes().path() {
        Some(path) => storage::tilde(path),
        None => "nowhere to save themes (no $HOME)".to_string(),
    };

    lines.extend([
        kit::heading("keys", ROW, current),
        kit::key("e", "edit your themes in $EDITOR", current),
        kit::key("r", "reload them from disk", current),
        kit::fact("file", file, current.dim, current),
        Line::from(""),
        kit::status(app.status(), current),
    ]);

    frame.render_widget(
        Paragraph::new(lines).block(super::panel(" theme ", current)),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_short_list_never_scrolls() {
        assert_eq!(offset(0, 5, 8), 0);
        assert_eq!(offset(4, 5, 8), 0);
    }

    #[test]
    fn scrolling_starts_only_when_the_selection_leaves_the_window() {
        assert_eq!(offset(7, 20, 8), 0);
        assert_eq!(offset(8, 20, 8), 1);
    }

    #[test]
    fn scrolling_stops_at_the_last_screenful() {
        assert_eq!(offset(19, 20, 8), 12);
    }

    #[test]
    fn a_list_shorter_than_the_window_doesnt_underflow() {
        assert_eq!(offset(0, 1, 8), 0);
        assert_eq!(offset(0, 0, 8), 0);
    }
}
