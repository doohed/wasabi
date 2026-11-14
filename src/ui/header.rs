use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;

/// Countdown and live WPM — or, in a word test, how many words are done.
///
/// A practice run also names the keys it is drilling, so the words being
/// strange-looking reads as the setting doing its job rather than as a broken
/// word list.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let clock = match app.remaining() {
        Some(remaining) => {
            let remaining = remaining.as_secs();
            format!("{:02}:{:02}", remaining / 60, remaining % 60)
        }
        // Words committed, not the one under the caret: the count says how
        // much is behind you, like the clock says how much is left.
        None => format!(
            "{}/{}",
            app.cursor_word.min(app.words.len()),
            app.words.len()
        ),
    };

    // `None` early in a test, when there is no honest figure to show yet.
    let wpm = match app.wpm() {
        Some(wpm) => format!("{wpm:.0} wpm"),
        None => "-- wpm".to_string(),
    };

    let practising = app.mode().modifiers().is_some_and(|m| m.practice);
    let focus = match app.focus() {
        _ if !practising => String::new(),
        [] => " · no weak keys yet".to_string(),
        keys => {
            let keys: Vec<String> = keys.iter().map(char::to_string).collect();
            format!(" · practising {}", keys.join(" "))
        }
    };

    let bar = Paragraph::new(format!("{clock} · {wpm}{focus}"))
        .centered()
        .style(Style::default().fg(app.theme().text));

    frame.render_widget(bar, area);
}
