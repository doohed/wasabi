use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::kit;
use crate::app::{code_rows, App};
use crate::typing::mode::Language;

/// Width of the picker, in columns.
pub const WIDTH: u16 = 40;
/// Columns the rows span, inside the margins.
const ROW: usize = kit::content(WIDTH);
/// The heading, one row per choice, a blank and the note under the list, the
/// panel's title, and a row of padding each side.
pub const HEIGHT: u16 = (Language::ALL.len() + 1) as u16 + 3 + 3;

/// What a row offers. `None` is the word test — the way back out.
fn label(choice: Option<Language>) -> String {
    match choice {
        None => "off".to_string(),
        Some(language) => language.name().to_string(),
    }
}

/// A word on what each row deals, beside the radio button.
fn detail(choice: Option<Language>) -> String {
    match choice {
        None => "back to words".to_string(),
        Some(language) => format!("{} snippets", language.name()),
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

    let mut lines = vec![kit::heading("language", ROW, theme)];

    lines.extend(code_rows().iter().enumerate().map(|(index, choice)| {
        let highlighted = index == app.menu_index;

        let mut left = kit::bar(highlighted, theme);
        left.push(Span::styled(
            format!("{:<10}", label(*choice)),
            kit::name(highlighted, theme),
        ));
        left.push(Span::styled(
            detail(*choice),
            Style::default().fg(theme.dim),
        ));

        // The dot marks what is in force, which is not necessarily the row
        // under the cursor.
        kit::spread(left, vec![kit::radio(*choice == chosen, theme)], ROW)
    }));

    lines.push(Line::from(""));
    lines.push(kit::note("indentation is typed for you", theme));

    let panel = Paragraph::new(lines).block(super::panel(" code ", theme));

    frame.render_widget(panel, area);
}
