use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use crate::theme::{Theme, THEMES};

/// Width of the picker, in columns.
pub const WIDTH: u16 = 40;
/// One row per theme, plus a row of padding top and bottom, plus the border.
pub const HEIGHT: u16 = THEMES.len() as u16 + 4;

/// A strip of the colours that theme actually uses, so a name isn't the only
/// thing to choose by.
fn swatch(theme: &Theme) -> Vec<Span<'static>> {
    [theme.text, theme.dim, theme.error, theme.accent, theme.good]
        .into_iter()
        .map(|colour| Span::styled("██", Style::default().fg(colour)))
        .collect()
}

/// The theme picker.
///
/// The rest of the interface is already drawn in the highlighted theme — the
/// preview is the app itself, not a sample inside this box.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let current = app.theme();
    let selected = app.theme_index();

    let rows: Vec<Line> = THEMES
        .iter()
        .enumerate()
        .map(|(index, theme)| {
            let highlighted = index == selected;

            let mut label = Style::default().fg(current.muted);
            if highlighted {
                label = label.add_modifier(Modifier::REVERSED);
            }

            let mut spans = vec![
                Span::raw(if highlighted { " › " } else { "   " }),
                Span::styled(format!(" {:<10}", theme.name), label),
                Span::raw("  "),
            ];
            spans.extend(swatch(theme));

            Line::from(spans)
        })
        .collect();

    let picker = Paragraph::new(rows).block(super::panel(" theme ", current));

    frame.render_widget(picker, area);
}
