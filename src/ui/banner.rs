use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use crate::storage;
use crate::theme::Theme;

/// Width of the banner box, in columns.
pub const WIDTH: u16 = 60;
/// Eleven content rows, plus the heading and a row of padding each side.
pub const HEIGHT: u16 = 14;

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

    // Off is the first thing to say: it explains every other row at once,
    // rather than leaving someone to wonder why art that "fits" isn't there.
    let (source, source_colour) = if app.banner_shown() {
        (source.to_string(), theme.accent)
    } else {
        (
            format!("off — {source} when you turn it back on"),
            theme.dim,
        )
    };

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

    // Read-only: the art's colour belongs to the theme now. Shown anyway, in
    // the colour itself, so someone looking for the old `c` key is told where
    // it went instead of finding nothing.
    let colour = Line::from(vec![
        Span::styled(
            format!(" {:<LABEL_WIDTH$}", "colour"),
            Style::default().fg(theme.dim),
        ),
        Span::styled("████", Style::default().fg(theme.banner)),
        Span::styled(
            format!("  from the {} theme", theme.name),
            Style::default().fg(theme.dim),
        ),
    ]);

    let mut lines = vec![
        field("showing", source, source_colour, theme),
        field("file", file, theme.accent, theme),
        field("size", size, size_colour, theme),
        colour,
        Line::from(""),
        key("e", "edit it in $EDITOR", theme),
        key(
            "b",
            if app.banner_shown() {
                "turn it off, and centre the test"
            } else {
                "turn it back on"
            },
            theme,
        ),
        key("r", "reload it from disk", theme),
        key("x", "remove it and use the built-in", theme),
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

    let panel = Paragraph::new(lines).block(super::panel(" banner ", theme));

    frame.render_widget(panel, area);
}
