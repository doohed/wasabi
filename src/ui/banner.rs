use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::kit;
use crate::app::App;
use crate::config::storage;

/// Width of the banner box, in columns.
pub const WIDTH: u16 = 60;
/// Columns the rows span, inside the margins.
const ROW: usize = kit::content(WIDTH);
/// Two headings, five facts, four keys, a blank and the status line, plus
/// the panel's title and a row of padding each side.
pub const HEIGHT: u16 = 13 + 3;

/// Where the art above the test comes from, and how to change it.
///
/// `screen_height` is the whole terminal, not this box: the useful thing to
/// tell someone about their art is whether it will actually show.
pub fn render(frame: &mut Frame, area: Rect, app: &App, screen_height: u16) {
    let theme = app.theme();
    let banner = app.banner();

    let source = if banner.is_custom() {
        "your own art"
    } else {
        "the built-in art"
    };

    // On or off first: it explains every other row at once, rather than
    // leaving someone to wonder why art that "fits" isn't there.
    let shown = kit::field(
        "showing",
        vec![
            kit::switch(app.banner_shown(), theme),
            Span::styled("  ·  ", Style::default().fg(theme.dim)),
            Span::styled(source, Style::default().fg(theme.muted)),
        ],
        theme,
    );

    let file = match banner.path() {
        Some(path) => storage::tilde(path),
        None => "nowhere to save (no $HOME)".to_string(),
    };

    // What the test screen needs on top of the art: the text, plus the header
    // and footer. Built from the same constants `ui::with_art` checks against,
    // so this can't claim art fits when it doesn't.
    let needed = super::rows_for_art(banner.height());
    let (size, size_colour) = if !app.banner_shown() {
        // Reporting a fit for art nobody is drawing would just be noise.
        (
            format!("{} × {}", banner.width(), banner.height()),
            theme.dim,
        )
    } else if needed <= screen_height {
        (
            format!("{} × {}", banner.width(), banner.height()),
            theme.accent,
        )
    } else {
        (
            // Kept short enough to fit the box: a truncated warning is worse
            // than a terse one.
            format!(
                "{} × {} — hidden, needs {needed} rows (have {screen_height})",
                banner.width(),
                banner.height()
            ),
            theme.warn,
        )
    };

    // Read-only: the art's colour belongs to the theme now (or, for the
    // built-in sushi, to the sushi). Shown anyway, in the colour itself, so
    // someone looking for the old `c` key is told where it went instead of
    // finding nothing.
    let colour = if banner.is_custom() {
        kit::field(
            "colour",
            vec![
                Span::styled("████", Style::default().fg(theme.banner)),
                Span::styled(
                    format!("  from the {} theme", theme.name),
                    Style::default().fg(theme.dim),
                ),
            ],
            theme,
        )
    } else {
        kit::field(
            "colour",
            vec![
                Span::styled("██", Style::default().fg(super::art::SALMON)),
                Span::styled("██", Style::default().fg(super::art::STRIPE)),
                Span::styled("██", Style::default().fg(theme.text)),
                Span::styled("  salmon on rice", Style::default().fg(theme.dim)),
            ],
            theme,
        )
    };

    let file_colour = if banner.is_custom() {
        theme.muted
    } else {
        theme.dim
    };

    let lines = vec![
        kit::heading("art", ROW, theme),
        shown,
        kit::fact("file", file, file_colour, theme),
        kit::fact("size", size, size_colour, theme),
        colour,
        kit::heading("keys", ROW, theme),
        kit::key("e", "edit it in $EDITOR", theme),
        kit::key(
            "b",
            if app.banner_shown() {
                "turn it off, and centre the test"
            } else {
                "turn it back on"
            },
            theme,
        ),
        kit::key("r", "reload it from disk", theme),
        kit::key("x", "remove it and use the built-in", theme),
        Line::from(""),
        kit::status(app.status(), theme),
    ];

    let panel = Paragraph::new(lines).block(super::panel(" banner ", theme));

    frame.render_widget(panel, area);
}
