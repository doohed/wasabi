use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::kit;
use crate::app::App;
use crate::config::storage;

/// Width of the words box, in columns.
pub const WIDTH: u16 = 60;
/// Columns the rows span, inside the margins.
const ROW: usize = kit::content(WIDTH);
/// Two headings, four facts, three keys, a blank and the status line, plus
/// the panel's title and a row of padding each side.
pub const HEIGHT: u16 = 11 + 3;

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
    let applied = match app.mode().label() {
        Some(label) => (label, theme.accent),
        None => (
            "none — the words as the list has them".to_string(),
            theme.dim,
        ),
    };

    let (source_colour, file_colour) = if wordlist.is_custom() {
        (theme.accent, theme.muted)
    } else {
        (theme.muted, theme.dim)
    };

    let lines = vec![
        kit::heading("source", ROW, theme),
        kit::fact("using", source, source_colour, theme),
        kit::fact("file", file, file_colour, theme),
        kit::fact("words", count, count_colour, theme),
        kit::fact("applied", applied.0, applied.1, theme),
        kit::heading("keys", ROW, theme),
        kit::key("e", "edit them in $EDITOR", theme),
        kit::key("r", "reload them from disk", theme),
        kit::key("x", "remove them and use the built-in list", theme),
        Line::from(""),
        kit::status(app.status(), theme),
    ];

    let panel = Paragraph::new(lines).block(super::panel(" words ", theme));

    frame.render_widget(panel, area);
}

/// A pool below this many words is worth mentioning.
const SHORT: usize = 10;
