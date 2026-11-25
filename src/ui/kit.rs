//! The pieces every panel is built from, so the settings menu and the screens
//! it opens read as one interface: ruled headings, a bar for the cursor,
//! dots for switches, keys as pills, and one place for a status message.
//!
//! Every line starts [`MARGIN`] columns in, then two columns the cursor bar
//! owns whether or not it is drawn, so rows with and without a cursor line up.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::config::theme::Theme;

/// Columns either side of a panel's content.
pub const MARGIN: usize = 2;
/// Columns the cursor bar takes, drawn or not.
const BAR: usize = 2;
/// Columns a field's label gets.
const LABEL: usize = 10;

/// The width of a panel's content, given the panel's width.
pub const fn content(width: u16) -> usize {
    width as usize - 2 * MARGIN
}

fn margin() -> Span<'static> {
    Span::raw(" ".repeat(MARGIN))
}

/// `── name ─────────` — a group's name, ruled off to `width` columns.
pub fn heading(name: &str, width: usize, theme: &Theme) -> Line<'static> {
    let rule = width.saturating_sub(Span::raw(name).width() + 4);
    Line::from(vec![
        margin(),
        Span::styled("── ", Style::default().fg(theme.dim)),
        Span::styled(
            name.to_string(),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {}", "─".repeat(rule)),
            Style::default().fg(theme.dim),
        ),
    ])
}

/// The left edge of a row: the margin, then the cursor bar or room for it.
pub fn bar(highlighted: bool, theme: &Theme) -> Vec<Span<'static>> {
    vec![
        margin(),
        if highlighted {
            Span::styled("▌ ", Style::default().fg(theme.accent))
        } else {
            Span::raw(" ".repeat(BAR))
        },
    ]
}

/// How a row's name is drawn: bright and bold under the cursor.
pub fn name(highlighted: bool, theme: &Theme) -> Style {
    if highlighted {
        Style::default().fg(theme.text).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.muted)
    }
}

/// `left` with `right` pushed against the far edge of `width` columns.
pub fn spread(
    mut left: Vec<Span<'static>>,
    right: Vec<Span<'static>>,
    width: usize,
) -> Line<'static> {
    let used: usize = left.iter().chain(&right).map(Span::width).sum();
    // `width` counts from the content's edge; the margin is extra.
    left.push(Span::raw(" ".repeat((width + MARGIN).saturating_sub(used))));
    left.extend(right);
    Line::from(left)
}

/// A labelled fact: `label` dim in a column of its own, then the value.
pub fn field(label: &str, value: Vec<Span<'static>>, theme: &Theme) -> Line<'static> {
    let mut spans = bar(false, theme);
    spans.push(Span::styled(
        format!("{label:<LABEL$}"),
        Style::default().fg(theme.dim),
    ));
    spans.extend(value);
    Line::from(spans)
}

/// A field whose value is one colour.
pub fn fact(label: &str, value: impl Into<String>, colour: Color, theme: &Theme) -> Line<'static> {
    field(
        label,
        vec![Span::styled(value.into(), Style::default().fg(colour))],
        theme,
    )
}

/// A key and what it does, the key drawn as a pill.
pub fn key(key: &str, what: &str, theme: &Theme) -> Line<'static> {
    let mut spans = bar(false, theme);
    spans.push(Span::styled(
        format!(" {key} "),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::REVERSED | Modifier::BOLD),
    ));
    spans.push(Span::styled(
        format!("  {what}"),
        Style::default().fg(theme.muted),
    ));
    Line::from(spans)
}

/// On or off, the way the settings menu draws a switch.
pub fn switch(on: bool, theme: &Theme) -> Span<'static> {
    if on {
        Span::styled("●  on", Style::default().fg(theme.accent))
    } else {
        Span::styled("○ off", Style::default().fg(theme.dim))
    }
}

/// One of a choice of one: filled when it is the choice.
pub fn radio(chosen: bool, theme: &Theme) -> Span<'static> {
    if chosen {
        Span::styled("●", Style::default().fg(theme.accent))
    } else {
        Span::styled("○", Style::default().fg(theme.dim))
    }
}

/// A quiet line of explanation, indented to the row names.
pub fn note(text: &str, theme: &Theme) -> Line<'static> {
    Line::from(Span::styled(
        format!("{}{text}", " ".repeat(MARGIN + BAR)),
        Style::default()
            .fg(theme.dim)
            .add_modifier(Modifier::ITALIC),
    ))
}

/// The last thing a key did, or a blank row: always a row, so a panel doesn't
/// change height when a message appears.
pub fn status(status: Option<&str>, theme: &Theme) -> Line<'static> {
    match status {
        Some(status) => Line::from(vec![
            margin(),
            Span::styled("› ", Style::default().fg(theme.accent)),
            Span::styled(
                status.to_string(),
                Style::default()
                    .fg(theme.muted)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]),
        None => Line::from(""),
    }
}
