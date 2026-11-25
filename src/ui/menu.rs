use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::kit;
use crate::app::{App, MenuItem, MenuKind, MENU, SECTIONS};

/// Width of the menu box, in columns.
pub const WIDTH: u16 = 40;
/// Columns the rows span, inside the margins.
const ROW: usize = kit::content(WIDTH);
/// Columns the row's name gets; the value is right-aligned in what is left.
const LABEL: usize = 14;
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

/// The right-hand side of a row: what the setting is now.
fn value(item: MenuItem, app: &App, live: bool, highlighted: bool) -> Vec<Span<'static>> {
    let theme = app.theme();
    let dim = Style::default().fg(theme.dim);

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
                    Span::styled(
                        cell,
                        if highlighted {
                            Style::default().fg(theme.muted)
                        } else {
                            dim
                        },
                    )
                }
            })
            .collect(),

        MenuKind::Switch => {
            let mut switch = kit::switch(app.menu_ticked(item), theme);
            if !live {
                switch.style = dim;
            }
            vec![switch]
        }

        // What is behind the screen, then a chevron to say there is one.
        MenuKind::Screen => {
            let (text, tint) = match item {
                MenuItem::Words if app.wordlist().is_custom() => {
                    ("custom".to_string(), theme.accent)
                }
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

    let mut name = kit::name(highlighted, theme);
    if !live {
        name = name.fg(if highlighted { theme.muted } else { theme.dim });
    }

    let mut left = kit::bar(highlighted, theme);
    left.push(Span::styled(format!("{:<LABEL$}", label(item)), name));

    kit::spread(left, value(item, app, live, highlighted), ROW)
}

/// The settings, grouped: what the test is, what you type, how it looks, and
/// how it has gone. Every row says what it is set to, so the menu doubles as a
/// summary of the test you are about to take.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();

    let mut lines: Vec<Line> = Vec::new();
    for (name, items) in SECTIONS {
        lines.push(kit::heading(name, ROW, theme));
        lines.extend(items.iter().map(|item| row(*item, app)));
    }

    let current = MENU[app.menu_index];
    let hint = if app.menu_live(current) {
        hint(current)
    } else {
        "words only — code tests ignore it"
    };

    lines.push(Line::from(""));
    lines.push(kit::note(hint, theme));

    let menu = Paragraph::new(lines).block(super::panel(" settings ", theme));

    frame.render_widget(menu, area);
}
