use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, MenuItem, MenuKind, MENU, SECTIONS};
use crate::config::theme::Theme;

/// Columns a row's contents span, from the cursor bar to the last value.
const ROW: usize = 36;
/// Columns the row's name gets; the value is right-aligned in what is left.
const LABEL: usize = 14;
/// Columns either side of the rows.
const MARGIN: usize = 2;

/// Width of the menu box, in columns.
pub const WIDTH: u16 = (ROW + 2 * MARGIN) as u16;
/// The rows and their headings, a rule and the hint under them, the panel's
/// title, and a row of padding each side.
pub const HEIGHT: u16 = (MENU.len() + SECTIONS.len()) as u16 + 2 + 3;

fn label(item: MenuItem) -> &'static str {
    match item {
        MenuItem::Time => "time",
        MenuItem::Count => "words",
        // Not "words": that would sit under the word-count row and read as
        // another length rather than a way into the pool they are drawn from.
        MenuItem::Words => "word list",
        MenuItem::Code => "code",
        MenuItem::Punctuation => "punctuation",
        MenuItem::Numbers => "numbers",
        MenuItem::Practice => "practice",
        MenuItem::Pace => "pace caret",
        MenuItem::Banner => "banner",
        MenuItem::Theme => "theme",
        MenuItem::Records => "records",
    }
}

/// One line on what the highlighted row is for, so the names can stay short.
fn hint(item: MenuItem) -> &'static str {
    match item {
        MenuItem::Time => "type until the clock runs out",
        MenuItem::Count => "type a fixed number of words",
        MenuItem::Words => "the pool the test draws words from",
        MenuItem::Code => "type real source code instead",
        MenuItem::Punctuation => "punctuate the words you type",
        MenuItem::Numbers => "mix the odd number in",
        MenuItem::Practice => "drill the keys you miss most",
        MenuItem::Pace => "a ghost caret at your best speed",
        MenuItem::Theme => "colours for the whole app",
        MenuItem::Banner => "the art above the test",
        MenuItem::Records => "personal bests and progress",
    }
}

/// `── text ─────────` — a group's name, ruled off to the width of the rows.
fn heading(name: &str, theme: &Theme) -> Line<'static> {
    let rule = ROW.saturating_sub(name.len() + 4);
    Line::from(vec![
        Span::raw(" ".repeat(MARGIN)),
        Span::styled("── ", Style::default().fg(theme.dim)),
        Span::styled(
            name.to_string(),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {}", "─".repeat(rule)), Style::default().fg(theme.dim)),
    ])
}

/// The right-hand side of a row: what the setting is now.
fn value(item: MenuItem, app: &App, live: bool, highlighted: bool) -> Vec<Span<'static>> {
    let theme = app.theme();
    let dim = Style::default().fg(theme.dim);
    let colour = |c: Color| Style::default().fg(if live { c } else { theme.dim });

    match item.kind() {
        // Every option across the row, the one in force as a filled pill.
        MenuKind::Choice => item
            .options()
            .iter()
            .map(|length| {
                let cell = format!(" {:>3} ", length.number());
                if *length == app.length() {
                    Span::styled(
                        cell,
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::REVERSED | Modifier::BOLD),
                    )
                } else {
                    Span::styled(cell, if highlighted { Style::default().fg(theme.muted) } else { dim })
                }
            })
            .collect(),

        MenuKind::Switch => {
            if app.menu_ticked(item) {
                vec![Span::styled("●  on", colour(theme.accent))]
            } else {
                vec![Span::styled("○ off", dim)]
            }
        }

        // What is behind the screen, then a chevron to say there is one.
        MenuKind::Screen => {
            let (text, tint) = match item {
                MenuItem::Words if app.wordlist().is_custom() => ("custom".to_string(), theme.accent),
                MenuItem::Words => ("built-in".to_string(), theme.muted),
                MenuItem::Code => match app.mode().language() {
                    Some(language) => (language.name().to_string(), theme.accent),
                    None => ("off".to_string(), theme.dim),
                },
                MenuItem::Theme => (app.theme().name.clone(), theme.muted),
                MenuItem::Banner if app.banner_shown() => ("on".to_string(), theme.muted),
                MenuItem::Banner => ("off".to_string(), theme.dim),
                MenuItem::Records => match app.records().best(&app.record_key()) {
                    Some(best) => (format!("best {:.0} wpm", best.wpm), theme.good),
                    None => ("no runs yet".to_string(), theme.dim),
                },
                _ => (String::new(), theme.muted),
            };
            let chevron = if highlighted { theme.accent } else { theme.dim };
            vec![
                Span::styled(text, Style::default().fg(tint)),
                Span::styled("  ›", Style::default().fg(chevron)),
            ]
        }
    }
}

fn row(item: MenuItem, app: &App) -> Line<'static> {
    let theme = app.theme();
    let highlighted = MENU[app.menu_index] == item;
    let live = app.menu_live(item);

    // A bar down the left edge and a brighter name mark the cursor; the
    // values carry their own colour, so nothing here needs to be reversed.
    let (bar, name) = match (highlighted, live) {
        (true, _) => (
            Span::styled("▌ ", Style::default().fg(theme.accent)),
            Style::default()
                .fg(if live { theme.text } else { theme.muted })
                .add_modifier(Modifier::BOLD),
        ),
        (false, true) => (Span::raw("  "), Style::default().fg(theme.muted)),
        (false, false) => (Span::raw("  "), Style::default().fg(theme.dim)),
    };

    let value = value(item, app, live, highlighted);
    let value_width: usize = value.iter().map(Span::width).sum();
    let gap = ROW.saturating_sub(2 + LABEL + value_width);

    let mut spans = vec![
        Span::raw(" ".repeat(MARGIN)),
        bar,
        Span::styled(format!("{:<LABEL$}", label(item)), name),
        Span::raw(" ".repeat(gap)),
    ];
    spans.extend(value);
    Line::from(spans)
}

/// The settings, grouped: what the test is, what you type, how it looks, and
/// how it has gone. Every row says what it is set to, so the menu doubles as a
/// summary of the test you are about to take.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();

    let mut lines: Vec<Line> = Vec::new();
    for (name, items) in SECTIONS {
        lines.push(heading(name, theme));
        lines.extend(items.iter().map(|item| row(*item, app)));
    }

    let current = MENU[app.menu_index];
    let hint = if app.menu_live(current) {
        hint(current)
    } else {
        "words only — code tests ignore it"
    };

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("{}{hint}", " ".repeat(MARGIN + 2)),
        Style::default()
            .fg(theme.dim)
            .add_modifier(Modifier::ITALIC),
    )));

    let menu = Paragraph::new(lines).block(super::panel(" settings ", theme));

    frame.render_widget(menu, area);
}
