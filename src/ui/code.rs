use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{code_rows, App};
use crate::typing::mode::Language;

/// Width of the picker, in columns.
pub const WIDTH: u16 = 34;
/// One row per choice, plus the heading, a row of padding each side, and the
/// note under the list.
pub const HEIGHT: u16 = Language::ALL.len() as u16 + 6;

/// What a row offers. `None` is the word test — the way back out.
fn label(choice: Option<Language>) -> String {
    match choice {
        None => "off — type words".to_string(),
        Some(language) => format!("{} code", language.name()),
    }
}

/// Where the test's code comes from: a language, or none of them.
///
/// Chosen with `enter` rather than applied as the highlight moves, unlike the
/// theme picker. Moving there is free and the interface is the preview; here
/// every step would deal a fresh test and throw away the one on screen.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();
    let chosen = app.mode().language();

    let mut lines: Vec<Line> = code_rows()
        .iter()
        .enumerate()
        .map(|(index, choice)| {
            let highlighted = index == app.menu_index;

            let mut style = Style::default().fg(theme.muted);
            if highlighted {
                style = style.add_modifier(Modifier::REVERSED);
            }

            // The tick marks what is in force, which is not necessarily the
            // row under the cursor.
            let tick = if *choice == chosen { " ✓" } else { "" };

            Line::from(vec![
                Span::raw(if highlighted { " › " } else { "   " }),
                Span::styled(format!(" {:<16}", label(*choice)), style),
                Span::styled(tick, Style::default().fg(theme.accent)),
            ])
        })
        .collect();

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "   indentation is typed for you",
        Style::default()
            .fg(theme.dim)
            .add_modifier(Modifier::ITALIC),
    )));

    let panel = Paragraph::new(lines).block(super::panel(" code ", theme));

    frame.render_widget(panel, area);
}
