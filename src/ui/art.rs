use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::config::banner::Banner;

/// The fish on the built-in sushi. Fixed rather than taken from the theme:
/// salmon is salmon-coloured whatever the rest of the interface looks like.
pub const SALMON: Color = Color::Rgb(250, 128, 114);

/// The lines of fat across the salmon: paler than the fish, but still pink
/// enough to read as part of it rather than as rice.
pub const STRIPE: Color = Color::Rgb(255, 196, 178);

/// Draw the banner. The built-in sushi is coloured from its map, with `rice`
/// for the rice; the user's own art is drawn all in `colour`.
pub fn render(frame: &mut Frame, area: Rect, banner: &Banner, colour: Color, rice: Color) {
    let art = match banner.colours() {
        Some(colours) => Paragraph::new(sushi(banner.art(), colours, rice)),
        None => Paragraph::new(banner.art()).style(Style::default().fg(colour)),
    };

    frame.render_widget(art.centered(), area);
}

/// Colour each character of `art` by the letter in the same place in
/// `colours`, joining runs of one colour into a single span.
fn sushi(art: &str, colours: &str, rice: Color) -> Vec<Line<'static>> {
    let colour_of = |letter: Option<char>| match letter {
        Some('s') => SALMON,
        Some('l') => STRIPE,
        // Rice, plus anything the map doesn't cover: blanks don't show, and a
        // map that has fallen out of step with the art should still draw.
        _ => rice,
    };

    let mut map = colours.lines();

    art.lines()
        .map(|line| {
            let mut letters = map.next().unwrap_or("").chars();
            let mut spans: Vec<Span<'static>> = Vec::new();
            let mut run = String::new();
            let mut run_colour = None;

            for ch in line.chars() {
                let letter = letters.next();
                // A blank takes whatever colour is running, so it doesn't
                // break a run into two spans for nothing.
                let colour = if ch == '\u{2800}' || ch == ' ' {
                    run_colour.unwrap_or_else(|| colour_of(letter))
                } else {
                    colour_of(letter)
                };

                if run_colour.is_some_and(|current| current != colour) {
                    spans.push(Span::styled(
                        std::mem::take(&mut run),
                        Style::default().fg(run_colour.unwrap_or(rice)),
                    ));
                }
                run_colour = Some(colour);
                run.push(ch);
            }

            if let Some(colour) = run_colour {
                spans.push(Span::styled(run, Style::default().fg(colour)));
            }

            Line::from(spans)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn runs(lines: &[Line]) -> Vec<Vec<(String, Option<Color>)>> {
        lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| (span.content.to_string(), span.style.fg))
                    .collect()
            })
            .collect()
    }

    #[test]
    fn each_character_takes_its_colour_from_the_map() {
        let lines = sushi("⣿⣿⣿\n⣿⣿⣿", "ssl\nrrs", Color::White);

        assert_eq!(
            runs(&lines),
            [
                vec![
                    ("⣿⣿".to_string(), Some(SALMON)),
                    ("⣿".to_string(), Some(STRIPE)),
                ],
                vec![
                    ("⣿⣿".to_string(), Some(Color::White)),
                    ("⣿".to_string(), Some(SALMON)),
                ],
            ]
        );
    }

    #[test]
    fn blanks_join_the_run_they_sit_in() {
        let lines = sushi("⣿⠀⣿", "s.s", Color::White);

        assert_eq!(runs(&lines), [vec![("⣿⠀⣿".to_string(), Some(SALMON))]]);
    }

    #[test]
    fn art_beyond_the_map_is_drawn_as_rice() {
        let lines = sushi("⣿⣿\n⣿", "s", Color::White);

        assert_eq!(
            runs(&lines),
            [
                vec![
                    ("⣿".to_string(), Some(SALMON)),
                    ("⣿".to_string(), Some(Color::White)),
                ],
                vec![("⣿".to_string(), Some(Color::White))],
            ]
        );
    }
}
