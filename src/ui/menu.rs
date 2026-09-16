use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, MenuItem, MENU};

/// Width of the menu box, in columns.
pub const WIDTH: u16 = 30;
/// A border and a row of padding top and bottom, around one row per item.
pub const HEIGHT: u16 = MENU.len() as u16 + 4;

fn label(item: MenuItem) -> String {
    match item {
        MenuItem::Duration(seconds) => format!("{seconds} seconds"),
        MenuItem::Banner => "banner".to_string(),
        MenuItem::Theme => "theme".to_string(),
        MenuItem::Records => "records".to_string(),
    }
}

/// The settings menu: test length, and a way into the records table.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();
    let active = app.duration_index();

    let rows: Vec<Line> = MENU
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let highlighted = index == app.menu_index;

            let mut style = Style::default().fg(theme.muted);
            if highlighted {
                style = style.add_modifier(Modifier::REVERSED);
            }

            // The tick marks the setting in force, which is not necessarily
            // the row under the cursor.
            let tick = if index == active { " ✓" } else { "" };

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
