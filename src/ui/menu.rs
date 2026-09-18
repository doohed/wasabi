use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, MenuItem, MENU};

/// Width of the menu box, in columns.
pub const WIDTH: u16 = 30;
/// One row per item, plus the heading and a row of padding each side.
pub const HEIGHT: u16 = MENU.len() as u16 + 3;

fn label(item: MenuItem) -> String {
    match item {
        MenuItem::Duration(seconds) => format!("{seconds} seconds"),
        MenuItem::Words => "words".to_string(),
        MenuItem::Code => "code".to_string(),
        MenuItem::Punctuation => "punctuation".to_string(),
        MenuItem::Numbers => "numbers".to_string(),
        MenuItem::Banner => "banner".to_string(),
        MenuItem::Theme => "theme".to_string(),
        MenuItem::Records => "records".to_string(),
    }
}

/// The settings menu: test length, and a way into the records table.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();

    let rows: Vec<Line> = MENU
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let highlighted = index == app.menu_index;

            let mut style = Style::default().fg(theme.muted);
            if highlighted {
                style = style.add_modifier(Modifier::REVERSED);
            }

            // The tick marks a setting in force, which is not necessarily the
            // row under the cursor — and there can be several, because the
            // modifiers are switches rather than a choice of one.
            let tick = if app.menu_ticked(*item) { " ✓" } else { "" };

            Line::from(vec![
                Span::raw(if highlighted { " › " } else { "   " }),
                Span::styled(format!(" {:<12}", label(*item)), style),
                Span::styled(tick, Style::default().fg(theme.accent)),
            ])
        })
        .collect();

    let menu = Paragraph::new(rows).block(super::panel(" settings ", theme));

    frame.render_widget(menu, area);
}
