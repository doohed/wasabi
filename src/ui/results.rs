use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use crate::theme::Theme;

/// Width of the results box, in columns.
pub const WIDTH: u16 = 44;
/// Six rows of score, plus the heading and a row of padding each side.
pub const HEIGHT: u16 = 9;

/// Width the labels are padded to. Every row is the same total width, so
/// centring the lines individually still leaves the columns aligned.
const LABEL_WIDTH: usize = 8;
const VALUE_WIDTH: usize = 6;

/// One `label  value` row, labels dim so the numbers carry the line.
fn stat(label: &str, value: String, theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{label:>LABEL_WIDTH$}  "),
            Style::default().fg(theme.dim),
        ),
        Span::styled(
            format!("{value:<VALUE_WIDTH$}"),
            Style::default().fg(theme.accent),
        ),
    ])
}

/// The score, shown in place of the text once the test is over.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();

    let lines = vec![
        Line::from(Span::styled(
            match app.wpm() {
                Some(wpm) => format!("{wpm:.0} wpm"),
                None => "-- wpm".to_string(),
            },
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
        )),
        // Always a row, even when there's nothing to celebrate, so the box
        // doesn't change height between runs.
        if app.is_new_best() {
            Line::from(Span::styled(
                "★ personal best",
                Style::default().fg(theme.good).add_modifier(Modifier::BOLD),
            ))
        } else {
            Line::from("")
        },
        Line::from(""),
        stat("accuracy", format!("{:.0}%", app.accuracy()), theme),
        stat(
            "time",
            format!("{:.0}s", app.elapsed().as_secs_f64()),
            theme,
        ),
        // Keystrokes split into right and wrong, which is the detail accuracy
        // rounds away.
        stat(
            "chars",
            format!("{}/{}", app.keystrokes() - app.mistakes(), app.keystrokes()),
            theme,
        ),
    ];

    // The title names the test length, so a score is never ambiguous about
    // which record it was competing for.
    let results = Paragraph::new(lines)
        .alignment(Alignment::Center)
        // Centred, unlike the other panels: this one's content is centred
        // too, and a heading hugging the left edge above it reads as a
        // mistake rather than a choice.
        .block(
            super::panel(format!(" {}s test ", app.duration().as_secs()), theme)
                .title_alignment(Alignment::Center),
        );

    frame.render_widget(results, area);
}
