use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::config::banner::Banner;

pub fn render(frame: &mut Frame, area: Rect, banner: &Banner, colour: Color) {
    let art = Paragraph::new(banner.art())
        .centered()
        .style(Style::default().fg(colour));

    frame.render_widget(art, area);
}
