use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, Screen};

/// The key hints, which are per-screen: every letter is test input, so the
/// only keys a typing test can spare are the ones listed here.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let keys = match app.screen {
        Screen::Test => " tab restart · esc menu · ctrl-c quit ",
        Screen::Menu => " ↑↓ move · enter select · esc back ",
        Screen::Banner => " e edit · r reload · x remove · esc back ",
        Screen::Theme => " ↑↓ preview · esc back ",
        Screen::Records => " esc back ",
    };

    let help = Paragraph::new(keys).style(Style::default().fg(app.theme().dim));
    frame.render_widget(help, area);
}
