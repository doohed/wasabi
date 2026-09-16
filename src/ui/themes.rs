use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use crate::storage;
use crate::theme::Theme;

/// Width of the picker, in columns.
pub const WIDTH: u16 = 58;

/// Most theme rows shown at once.
///
/// Someone with thirty themes gets a scrolling list rather than a box taller
/// than their terminal.
const MAX_ROWS: usize = 8;

/// Rows below the list: a blank, two key hints, a blank, the file path, and
/// the status line.
const FOOTER_ROWS: u16 = 6;

/// How many theme rows are on screen.
fn visible(app: &App) -> usize {
    app.themes().all().len().min(MAX_ROWS)
}

/// Height the picker needs, which depends on how many themes exist.
pub fn height(app: &App) -> u16 {
    // The rows, the footer, a row of padding top and bottom, and the border.
    visible(app) as u16 + FOOTER_ROWS + 4
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

/// A strip of the colours that theme actually uses, so a name isn't the only
/// thing to choose by.
fn swatch(theme: &Theme) -> Vec<Span<'static>> {
    [
        theme.text,
        theme.dim,
        theme.error,
        theme.accent,
        theme.banner,
    ]
    .into_iter()
    .map(|colour| Span::styled("██", Style::default().fg(colour)))
    .collect()
}

fn key(key: &str, what: &str, theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!(" {key}  "), Style::default().fg(theme.accent)),
        Span::styled(what.to_string(), Style::default().fg(theme.muted)),
    ])
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

    let mut lines: Vec<Line> = all
        .iter()
        .enumerate()
        .skip(first)
        .take(visible)
        .map(|(index, theme)| {
            let highlighted = index == selected;

            let mut label = Style::default().fg(current.muted);
            if highlighted {
                label = label.add_modifier(Modifier::REVERSED);
            }

            let mut spans = vec![
                Span::raw(if highlighted { " › " } else { "   " }),
                Span::styled(format!(" {:<10}", theme.name), label),
                Span::raw("  "),
            ];
            spans.extend(swatch(theme));
            // Marks a theme as one of the user's own, so it's obvious which
            // rows come from their file.
            if theme.custom {
                spans.push(Span::styled("  ·", Style::default().fg(current.accent)));
            }

            Line::from(spans)
        })
        .collect();

    lines.push(Line::from(""));
    lines.push(key("e", "edit your themes in $EDITOR", current));
    lines.push(key("r", "reload them from disk", current));
    lines.push(Line::from(""));

    // Shown so that someone with no $EDITOR still knows which file to open.
    lines.push(Line::from(Span::styled(
        match app.themes().path() {
            Some(path) => format!(" {}", storage::tilde(path)),
            None => " nowhere to save themes (no $HOME)".to_string(),
        },
        Style::default().fg(current.dim),
    )));

    // Always a row, so the box doesn't change height when a message appears.
    lines.push(match app.status() {
        Some(status) => Line::from(Span::styled(
            format!(" {status}"),
            Style::default()
                .fg(current.muted)
                .add_modifier(Modifier::ITALIC),
        )),
        None => Line::from(""),
    });

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
