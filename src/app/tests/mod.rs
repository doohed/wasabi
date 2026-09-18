//! Tests for [`super::App`].
//!
//! A folder rather than an inline `mod tests` only for length: these are unit
//! tests of a private-by-default type, and reach its private fields through
//! `use super::*` exactly as an inline module would.
//!
//! Split by what they are about, not by which method they call — a test that
//! turns on punctuation and then checks what got filed belongs with the
//! setting that caused it. The fixtures every file shares live here.

use super::*;
use crate::typing::mode::Language;

mod input;
mod scores;
mod settings;
mod stats;

/// An app over `targets`, caret at the first word.
///
/// Built on an in-memory `Records` so a finished test in a test can't
/// write to the user's real records file. Every test goes through here.
fn app(targets: &[&str]) -> App {
    App {
        words: targets.iter().copied().map(Word::new).collect(),
        // In-memory records and a pathless banner, so no test can reach
        // the user's real files. Every test goes through here.
        ..App::detached()
    }
}

/// Where `item` sits in the menu. Positions shift when rows are added, so
/// tests ask rather than assume.
fn menu_row(item: MenuItem) -> usize {
    MENU.iter().position(|row| *row == item).expect("menu row")
}

fn type_str(app: &mut App, input: &str) {
    for c in input.chars() {
        match c {
            ' ' => app.type_space(),
            c => app.type_char(c),
        }
    }
}

/// `(cursor_word, cursor_char)` — the caret, as the renderer sees it.
fn caret(app: &App) -> (usize, usize) {
    (app.cursor_word, app.cursor_char())
}

/// The row a test at `seconds` is filed under, in `mode`.
///
/// Derived rather than written out: the key carries a scoring version, and
/// bumping that shouldn't send every test in this module looking for a row
/// that moved.
fn key(mode: Mode, seconds: u64) -> String {
    mode.key(seconds)
}

/// The speeds of every run filed at `key`, oldest first.
fn history(app: &App, key: &str) -> Vec<f64> {
    app.history().at(key).iter().map(|run| run.wpm).collect()
}
