use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;

/// Countdown and live WPM.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let remaining = app.remaining().as_secs();
    let clock = format!("{:02}:{:02}", remaining / 60, remaining % 60);

    // `None` early in a test, when there is no honest figure to show yet.
    let wpm = match app.wpm() {
        Some(wpm) => format!("{wpm:.0} wpm"),
        None => "-- wpm".to_string(),
    };

    let bar = Paragraph::new(format!("{clock} · {wpm}"))
        .centered()
        .style(Style::default().fg(app.theme().text));

    frame.render_widget(bar, area);
}
