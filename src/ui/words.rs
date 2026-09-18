use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use crate::storage;
use crate::theme::Theme;

/// Width of the words box, in columns.
pub const WIDTH: u16 = 60;
/// Ten content rows, plus the heading and a row of padding each side.
pub const HEIGHT: u16 = 13;

const LABEL_WIDTH: usize = 8;

fn field(label: &str, value: String, colour: Color, theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!(" {label:<LABEL_WIDTH$}"),
            Style::default().fg(theme.dim),
        ),
        Span::styled(value, Style::default().fg(colour)),
    ])
}

fn key(key: &str, what: &str, theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!(" {key}  "), Style::default().fg(theme.accent)),
        Span::styled(what.to_string(), Style::default().fg(theme.muted)),
    ])
}

/// Where the test's words come from, and how to change them.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();
    let wordlist = app.wordlist();

    let source = if wordlist.is_custom() {
        "your own words"
    } else {
        "the built-in list"
    };

    let file = match wordlist.path() {
        Some(path) => storage::tilde(path),
        None => "nowhere to save (no $HOME)".to_string(),
    };

    // A pool shorter than a handful of words makes a test that repeats itself
    // every line. It still works, and it is still what you asked for, so this
    // says so rather than refusing it.
    let (count, count_colour) = if wordlist.len() < SHORT {
        (
            format!("{} — short enough to repeat a lot", wordlist.len()),
            theme.warn,
        )
    } else {
        (wordlist.len().to_string(), theme.accent)
    };

    // The modifiers live in the menu, not here, but they decide what you
    // actually type — so the row that says where the words come from also says
    // what happens to them on the way.
    let applied = match app.modifiers().label() {
        Some(label) => (label, theme.accent),
        None => (
            "none — the words exactly as the list has them".to_string(),
            theme.dim,
        ),
    };

    let mut lines = vec![
        field("using", source.to_string(), theme.accent, theme),
        field("file", file, theme.accent, theme),
        field("words", count, count_colour, theme),
        field("applied", applied.0, applied.1, theme),
        Line::from(""),
        key("e", "edit them in $EDITOR", theme),
        key("r", "reload them from disk", theme),
        key("x", "remove them and use the built-in list", theme),
        Line::from(""),
    ];

    if let Some(status) = app.status() {
        lines.push(Line::from(Span::styled(
            format!(" {status}"),
            Style::default()
                .fg(theme.muted)
                .add_modifier(Modifier::ITALIC),
        )));
    }

    let panel = Paragraph::new(lines).block(super::panel(" words ", theme));

    frame.render_widget(panel, area);
}

/// A pool below this many words is worth mentioning.
const SHORT: usize = 10;
