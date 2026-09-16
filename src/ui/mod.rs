mod art;
mod banner;
mod footer;
mod header;
mod menu;
mod records;
mod results;
mod themes;
mod typing;

use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{Block, Padding};
use ratatui::Frame;

use crate::app::{App, Screen};
use crate::theme::Theme;

/// Maximum width of the typing column, in terminal columns.
const CONTENT_WIDTH: u16 = 72;
/// Rows reserved for the text, which is exactly the number of lines `typing`
/// scrolls through — no border to pay for.
const TEXT_HEIGHT: u16 = 3;
/// The stats bar: one unadorned line.
const HEADER_HEIGHT: u16 = 1;
/// The key hints: one unadorned line.
const FOOTER_HEIGHT: u16 = 1;
/// Blank rows between the header and the top of the art.
const ART_GAP: u16 = 1;
/// Blank rows between the bottom of the art and the text.
const TEXT_GAP: u16 = 4;
/// Everything the screen owes to something other than the body.
const CHROME_HEIGHT: u16 = HEADER_HEIGHT + FOOTER_HEIGHT;

/// Body rows needed to show `art_height` rows of art above `content_height`
/// rows of content — the gaps included, because they are real rows.
fn body_rows_for_art(art_height: u16, content_height: u16) -> u16 {
    ART_GAP + art_height + TEXT_GAP + content_height
}

/// Terminal rows needed to show `art_height` rows of art above the test.
///
/// The banner screen's answer to "will my art show?", derived from the same
/// arithmetic the layout uses so the two can't drift apart.
pub(super) fn rows_for_art(art_height: u16) -> u16 {
    body_rows_for_art(art_height, TEXT_HEIGHT) + CHROME_HEIGHT
}

/// A panel in the current palette: a heading, and room to breathe.
///
/// No border. A box drawn round every overlay competes with the text for
/// attention, and in a typing test the text has to win; the accented heading
/// and the padding are enough to say where a panel starts. A borderless block
/// still puts its title on the first row, so this costs one row rather than
/// the two a frame would.
fn panel(title: impl Into<String>, theme: &Theme) -> Block<'static> {
    Block::new()
        .title(Span::styled(
            title.into(),
            Style::default().fg(theme.accent),
        ))
        .padding(Padding::vertical(1))
}

pub fn draw(frame: &mut Frame, app: &App) {
    // Vertical split of the whole screen.
    let [header_area, body_area, footer_area] = Layout::vertical([
        Constraint::Length(HEADER_HEIGHT),
        Constraint::Fill(1), // everything left over
        Constraint::Length(FOOTER_HEIGHT),
    ])
    .areas(frame.area());

    header::render(frame, header_area, app);

    match app.screen {
        Screen::Test => test(frame, body_area, app),
        Screen::Menu => menu::render(frame, centred(body_area, menu::WIDTH, menu::HEIGHT), app),
        Screen::Banner => banner::render(
            frame,
            centred(body_area, banner::WIDTH, banner::HEIGHT),
            app,
            frame.area().height,
        ),
        Screen::Theme => themes::render(
            frame,
            centred(body_area, themes::WIDTH, themes::height(app)),
            app,
        ),
        Screen::Records => records::render(
            frame,
            centred(body_area, records::WIDTH, records::HEIGHT),
            app,
        ),
    }

    footer::render(frame, footer_area, app);
}

/// The test itself: the text while the clock runs, the score once it stops.
fn test(frame: &mut Frame, area: Rect, app: &App) {
    let (width, height) = if app.is_over() {
        (results::WIDTH, results::HEIGHT)
    } else {
        (CONTENT_WIDTH, TEXT_HEIGHT)
    };

    // With art above it the content hangs from the art, a fixed gap below;
    // with no art the body is empty, so centring is what looks deliberate.
    let (area, vertical) = match with_art(frame, area, app, height) {
        Some(below_art) => (below_art, Flex::Start),
        None => (area, Flex::Center),
    };

    let area = column(area, width, height, vertical);

    if app.is_over() {
        results::render(frame, area, app);
        return;
    }

    // No border, so this area *is* the text area — the width passed to the
    // wrapping logic is the width the text gets.
    typing::render(frame, area, app);
}

/// Draw the art across the top of `area`, and return what is left below it for
/// `content_height` rows of content. `None` when there was no room to draw it.
///
/// The art is the first thing dropped when the terminal is small: a typing
/// test with no text box is useless, one with no picture is merely plainer.
fn with_art(frame: &mut Frame, area: Rect, app: &App, content_height: u16) -> Option<Rect> {
    if !app.banner_shown() {
        return None;
    }

    let banner = app.banner();

    if area.width < banner.width()
        || area.height < body_rows_for_art(banner.height(), content_height)
    {
        return None;
    }

    let [_, art_area, _, rest] = Layout::vertical([
        Constraint::Length(ART_GAP),
        Constraint::Length(banner.height()),
        Constraint::Length(TEXT_GAP),
        Constraint::Fill(1),
    ])
    .areas(area);

    art::render(frame, art_area, banner, app.banner_colour());
    Some(rest)
}

/// A `width` × `height` box, always centred horizontally, placed vertically by
/// `vertical`.
///
/// `Flex::Center` pushes all the leftover space to the outside, so a single
/// constraint lands in the middle; `Flex::Start` pins it to the top instead.
/// `Max` rather than `Length` so a small terminal degrades to its own size
/// instead of overflowing.
fn column(area: Rect, width: u16, height: u16, vertical: Flex) -> Rect {
    let [column] = Layout::horizontal([Constraint::Max(width)])
        .flex(Flex::Center)
        .areas(area);

    let [box_] = Layout::vertical([Constraint::Max(height)])
        .flex(vertical)
        .areas(column);

    box_
}

/// A `width` × `height` box in the middle of `area`.
fn centred(area: Rect, width: u16, height: u16) -> Rect {
    column(area, width, height, Flex::Center)
}
