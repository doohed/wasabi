use chrono::Local;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, Screen};

/// Columns the clock costs: `HH:MM`, plus the space that keeps it off the
/// right edge the way the hints are kept off the left one.
const CLOCK_WIDTH: u16 = 6;

/// The key hints, which are per-screen: every letter is test input, so the
/// only keys a typing test can spare are the ones listed here. The wall clock
/// sits in the opposite corner, since a test worth taking is one you lose
/// track of the time in.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let keys = match app.screen {
        Screen::Test => " tab restart · esc menu · ctrl-c quit ",
        Screen::Menu => " ↑↓ move · enter select · esc back ",
        Screen::Words => " e edit · r reload · x remove · esc back ",
        Screen::Code => " ↑↓ move · enter select · esc back ",
        Screen::Banner => " e edit · b on/off · r reload · x remove · esc back ",
        Screen::Theme => " ↑↓ preview · e edit · r reload · esc back ",
        Screen::Records => " esc back ",
    };

    let style = Style::default().fg(app.theme().dim);
    let (keys_area, clock_area) = split(area, Line::from(keys).width() as u16);

    frame.render_widget(Paragraph::new(keys).style(style), keys_area);

    if let Some(clock_area) = clock_area {
        // 24-hour, so the clock is the same width all day and the corner
        // never shifts under you mid-word.
        let now = Local::now().format("%H:%M ").to_string();
        frame.render_widget(Paragraph::new(now).right_aligned().style(style), clock_area);
    }
}

/// Split the footer into the hints and the clock's corner, or hand the whole
/// row to the hints when the two won't fit side by side.
///
/// The hints are how you get off the screen; the time is only nice to know.
/// So on a narrow terminal the clock is what goes, rather than the clock
/// truncating a hint — the same bargain the art strikes in [`super::with_art`].
fn split(area: Rect, keys_width: u16) -> (Rect, Option<Rect>) {
    if area.width < keys_width + CLOCK_WIDTH {
        return (area, None);
    }

    let [keys, clock] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(CLOCK_WIDTH)]).areas(area);

    (keys, Some(clock))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn footer(width: u16) -> Rect {
        Rect::new(0, 0, width, 1)
    }

    #[test]
    fn the_clock_takes_the_right_hand_corner() {
        let (keys, clock) = split(footer(40), 10);
        let clock = clock.expect("room for both");

        assert_eq!(keys.width, 40 - CLOCK_WIDTH);
        assert_eq!(clock.right(), 40);
        assert_eq!(clock.width, CLOCK_WIDTH);
    }

    #[test]
    fn the_hints_keep_every_column_they_need() {
        // Exactly the hints plus the clock: the tightest fit that still works.
        let (keys, clock) = split(footer(16), 10);

        assert!(clock.is_some());
        assert!(keys.width >= 10);
    }

    #[test]
    fn the_clock_goes_before_a_hint_does() {
        let (keys, clock) = split(footer(15), 10);

        assert_eq!(clock, None);
        assert_eq!(keys, footer(15)); // the hints keep the whole row
    }
}
