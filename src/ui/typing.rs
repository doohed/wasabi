use std::ops::Range;

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use crate::config::theme::Theme;
use crate::typing::word::{CharState, Word};

/// The single place where a character state becomes a colour.
///
/// Everything that draws text goes through here, so a palette swap is one
/// lookup rather than a hunt through the renderer.
fn style_for(state: CharState, theme: &Theme) -> Style {
    let colour = match state {
        CharState::Correct => theme.text,
        CharState::Incorrect => theme.error,
        CharState::Extra => theme.extra,
        CharState::Untyped => theme.dim,
    };
    Style::default().fg(colour)
}

/// Mark the character the user is about to type.
///
/// `REVERSED` rather than a real terminal cursor: it draws a solid block even
/// on a space, and it can't drift out of sync with the text, because it *is*
/// the text.
fn caret(style: Style) -> Style {
    style.add_modifier(Modifier::REVERSED)
}

/// Mark where a run at your personal best would be.
///
/// Underlined rather than reversed, so it can never be mistaken for the caret
/// you are actually typing at. Ahead of you it sits on text you haven't typed,
/// which is drawn dim — so it takes the accent there, or it would be a dim
/// underline under dim text. Behind you it keeps the colour of what you typed,
/// because whether that was right matters more than where the ghost is.
fn pace(style: Style, state: CharState, theme: &Theme) -> Style {
    let style = style.add_modifier(Modifier::UNDERLINED);

    match state {
        CharState::Untyped => style.fg(theme.accent),
        _ => style,
    }
}

/// Where `chars` scoring characters into the text falls: a word, and an
/// offset into its target. An offset equal to the target's length is the
/// space after the word. `None` past the end of the text.
///
/// Counted over targets rather than what was typed, and a space per word —
/// the same characters [`App::correct_chars`] credits, so a pace caret level
/// with your caret means you are typing at your best.
fn locate(words: &[Word], mut chars: usize) -> Option<(usize, usize)> {
    for (index, word) in words.iter().enumerate() {
        let len = word.target.chars().count();
        if chars <= len {
            return Some((index, chars));
        }
        chars -= len + 1;
    }

    None
}

/// Columns a word occupies on screen.
///
/// The *displayed* width, which is the longer of the target and what was typed
/// — an overtyped word is wider than its target, and the wrap has to know.
fn display_width(word: &Word) -> usize {
    word.target.chars().count().max(word.typed.chars().count())
}

/// Columns of indentation drawn before a word, which is none unless the word
/// begins a line of code.
fn indent_width(word: &Word) -> usize {
    word.indent.unwrap_or(0) as usize
}

/// Break the word list into lines that fit `width` columns.
///
/// Greedy, and words are never split: the returned ranges partition
/// `0..words.len()` in order. Kept separate from rendering so the geometry can
/// be tested without a terminal.
///
/// A word that begins a line of code starts one here whatever is left over on
/// the current line — code that reflowed to fill the column would stop looking
/// like code, which is the only reason to type it.
fn wrap(words: &[Word], width: u16) -> Vec<Range<usize>> {
    let width = width as usize;
    if width == 0 {
        return Vec::new();
    }

    let mut lines = Vec::new();
    let mut start = 0;
    let mut used = 0;

    for (index, word) in words.iter().enumerate() {
        let opens_line = index == start;
        // The first word on a line is preceded by its indent; every other one
        // by a space.
        let needed = display_width(word) + if opens_line { indent_width(word) } else { 1 };

        // `!opens_line` keeps a word too long for the whole line on a line of
        // its own rather than looping forever on an empty one.
        let broken = !opens_line && (word.indent.is_some() || used + needed > width);

        if broken {
            lines.push(start..index);
            start = index;
            used = indent_width(word) + display_width(word);
        } else {
            used += needed;
        }
    }

    if start < words.len() {
        lines.push(start..words.len());
    }

    lines
}

/// The first line to draw, so the caret's line sits in the middle of the
/// `visible` rows.
///
/// Parking the caret in the middle keeps a line of what you just typed above
/// and a line of what's coming below, and the text scrolls under a caret that
/// doesn't move. Clamped at both ends: no scrolling until the caret has moved
/// past the top band, and never past the last screenful.
fn scroll_offset(caret_line: usize, total_lines: usize, visible: usize) -> usize {
    let band = visible.saturating_sub(1) / 2;
    let last_offset = total_lines.saturating_sub(visible);

    caret_line.saturating_sub(band).min(last_offset)
}

/// Render the words in `range` as one styled line, with the pace caret at
/// `ghost` if it falls on it.
fn render_line(app: &App, range: Range<usize>, ghost: Option<(usize, usize)>) -> Line<'static> {
    let theme = app.theme();
    let mut spans: Vec<Span> = Vec::new();

    // Lining code up is the editor's job in real life, so the indent is drawn
    // rather than typed. It carries no state: there is nothing here to get
    // right or wrong.
    let indent = indent_width(&app.words[range.start]);
    if indent > 0 {
        spans.push(Span::raw(" ".repeat(indent)));
    }

    // Set when the caret belongs *after* the last character of a word — the
    // word is fully typed (or overtyped) and the next keystroke should be the
    // space. The separator span drawn on the next iteration claims it.
    let mut caret_on_separator = false;
    // The same, for the pace caret.
    let mut ghost_on_separator = false;

    for index in range.clone() {
        // Separator between words. The space is not part of either word, so a
        // mistyped space can't currently be marked as an error — a limitation
        // to revisit if you want that behaviour.
        if index != range.start {
            let mut style = style_for(CharState::Untyped, theme);
            if caret_on_separator {
                style = caret(style);
                caret_on_separator = false;
            }
            if ghost_on_separator {
                style = pace(style, CharState::Untyped, theme);
                ghost_on_separator = false;
            }
            spans.push(Span::styled(" ", style));
        }

        let word = &app.words[index];
        let states = word.char_states();
        // `Some(n)` only on the word being typed: n is the caret's offset.
        let caret_at = (index == app.cursor_word).then(|| app.cursor_char());
        // The ghost goes by the target, so an overtyped word's extras never
        // hold it: past the target's end, it is on the space.
        let target_len = word.target.chars().count();
        let ghost_at = ghost.and_then(|(at, offset)| (at == index).then_some(offset));

        for (offset, (character, state)) in states.iter().enumerate() {
            let mut style = style_for(*state, theme);
            if caret_at == Some(offset) {
                style = caret(style);
            }
            if ghost_at == Some(offset) && offset < target_len {
                style = pace(style, *state, theme);
            }
            spans.push(Span::styled(character.to_string(), style));
        }

        if caret_at.is_some_and(|offset| offset >= states.len()) {
            caret_on_separator = true;
        }
        if ghost_at.is_some_and(|offset| offset >= target_len) {
            ghost_on_separator = true;
        }
    }

    // The caret spilled off the end of the last word on this line: draw it
    // here rather than at the start of the next line, which is where the space
    // would have gone had the line had room for it.
    if caret_on_separator || ghost_on_separator {
        let mut style = style_for(CharState::Untyped, theme);
        if caret_on_separator {
            style = caret(style);
        }
        if ghost_on_separator {
            style = pace(style, CharState::Untyped, theme);
        }
        spans.push(Span::styled(" ", style));
    }

    Line::from(spans)
}

/// Draw the visible window of the test text, with the caret on the next
/// character to be typed.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let lines = wrap(&app.words, area.width);

    // Which line holds the caret. `position` rather than arithmetic because
    // lines hold a variable number of words.
    let caret_line = lines
        .iter()
        .position(|range| range.contains(&app.cursor_word))
        .unwrap_or(0);

    // How many lines fit is the box's business, not a constant here: a taller
    // terminal shows more of the test rather than leaving the room empty.
    let visible = area.height as usize;
    let offset = scroll_offset(caret_line, lines.len(), visible);
    let ghost = app.pace().and_then(|chars| locate(&app.words, chars));

    let rows: Vec<Line> = lines
        .into_iter()
        .skip(offset)
        .take(visible)
        .map(|range| render_line(app, range, ghost))
        .collect();

    frame.render_widget(Paragraph::new(rows), area);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(targets: &[&str]) -> Vec<Word> {
        targets.iter().copied().map(Word::new).collect()
    }

    #[test]
    fn everything_on_one_line_when_it_fits() {
        let words = words(&["ab", "cd"]);
        assert_eq!(wrap(&words, 5), vec![0..2]);
    }

    #[test]
    fn the_separating_space_counts_towards_the_width() {
        // "ab cd" is 5 columns; at 4 the second word has to move down.
        let words = words(&["ab", "cd"]);
        assert_eq!(wrap(&words, 4), vec![0..1, 1..2]);
    }

    #[test]
    fn a_trailing_space_is_not_required() {
        // Exactly 5 columns of text in 5 columns of width: no wrap.
        let words = words(&["ab", "cd"]);
        assert_eq!(wrap(&words, 5), vec![0..2]);
    }

    #[test]
    fn wrapping_continues_across_several_lines() {
        let words = words(&["aa", "bb", "cc", "dd"]);
        assert_eq!(wrap(&words, 5), vec![0..2, 2..4]);
    }

    #[test]
    fn overtyping_a_word_widens_it() {
        let mut words = words(&["ab", "cd"]);
        words[0].typed = "abxx".to_string(); // now 4 columns wide

        assert_eq!(wrap(&words, 5), vec![0..1, 1..2]);
    }

    #[test]
    fn a_word_wider_than_the_line_gets_a_line_to_itself() {
        let words = words(&["a", "enormous", "b"]);
        assert_eq!(wrap(&words, 3), vec![0..1, 1..2, 2..3]);
    }

    /// A line of code: the first word carries the indent.
    fn code_line(indent: u16, targets: &[&str]) -> Vec<Word> {
        targets
            .iter()
            .enumerate()
            .map(|(position, target)| match position {
                0 => Word::at_indent(target, indent),
                _ => Word::new(target),
            })
            .collect()
    }

    #[test]
    fn a_line_of_code_starts_a_line_however_much_room_is_left() {
        let mut words = code_line(0, &["fn", "f()", "{"]);
        words.extend(code_line(4, &["let", "x", "=", "1;"]));
        words.extend(code_line(0, &["}"]));

        // Room for all of it on one line, and it still breaks where the code
        // breaks — reflowed code stops looking like code.
        assert_eq!(wrap(&words, 80), vec![0..3, 3..7, 7..8]);
    }

    #[test]
    fn indentation_takes_up_room_on_its_line() {
        // "    let x" is nine columns, so eight is not enough for two words.
        let words = code_line(4, &["let", "x"]);

        assert_eq!(wrap(&words, 8), vec![0..1, 1..2]);
        assert_eq!(wrap(&words, 9), vec![0..2]);
    }

    #[test]
    fn a_long_line_of_code_still_wraps() {
        // The break is forced *into* new lines, never out of wrapping: a line
        // wider than the column has to go somewhere.
        let words = code_line(0, &["aaaa", "bbbb", "cccc"]);

        assert_eq!(wrap(&words, 9), vec![0..2, 2..3]);
    }

    #[test]
    fn the_pace_caret_starts_on_the_first_character() {
        assert_eq!(locate(&words(&["cat", "dog"]), 0), Some((0, 0)));
    }

    #[test]
    fn the_pace_caret_counts_the_space_after_each_word() {
        let words = words(&["cat", "dog"]);

        assert_eq!(locate(&words, 2), Some((0, 2)));
        assert_eq!(locate(&words, 3), Some((0, 3)), "the space after cat");
        assert_eq!(locate(&words, 4), Some((1, 0)));
        assert_eq!(locate(&words, 7), Some((1, 3)));
    }

    #[test]
    fn the_pace_caret_past_the_end_is_nowhere() {
        assert_eq!(locate(&words(&["cat"]), 5), None);
    }

    #[test]
    fn zero_width_wraps_to_nothing() {
        assert!(wrap(&words(&["a"]), 0).is_empty());
    }

    #[test]
    fn no_scrolling_until_the_caret_leaves_the_top_band() {
        assert_eq!(scroll_offset(0, 10, 3), 0);
        assert_eq!(scroll_offset(1, 10, 3), 0);
        assert_eq!(scroll_offset(2, 10, 3), 1);
    }

    #[test]
    fn scrolling_stops_at_the_last_screenful() {
        // 5 lines, 3 visible: the furthest the window can start is line 2.
        assert_eq!(scroll_offset(4, 5, 3), 2);
        assert_eq!(scroll_offset(9, 5, 3), 2);
    }

    #[test]
    fn a_test_shorter_than_the_box_never_scrolls() {
        assert_eq!(scroll_offset(1, 2, 3), 0);
    }

    #[test]
    fn a_taller_box_keeps_the_caret_centred() {
        // 5 visible rows: the caret band is row 2.
        assert_eq!(scroll_offset(2, 20, 5), 0);
        assert_eq!(scroll_offset(3, 20, 5), 1);
    }

    #[test]
    fn a_one_line_box_scrolls_every_line() {
        assert_eq!(scroll_offset(0, 20, 1), 0);
        assert_eq!(scroll_offset(4, 20, 1), 4);
    }
}
