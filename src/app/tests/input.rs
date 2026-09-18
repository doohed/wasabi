//! Typing, deleting, and where the caret goes.
//!
//! Fixtures come from [`super`], which is where every test in this module
//! builds its `App` — an in-memory one, so no test can reach the user's real
//! files.

use super::*;

// -- input ------------------------------------------------------------

#[test]
fn characters_stay_in_the_current_word() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "ca");

    assert_eq!(app.words[0].typed, "ca");
    assert_eq!(app.words[1].typed, "");
    assert_eq!(caret(&app), (0, 2));
}

#[test]
fn space_commits_the_word() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cat d");

    assert_eq!(app.words[0].typed, "cat");
    assert_eq!(app.words[1].typed, "d");
    assert_eq!(caret(&app), (1, 1));
}

#[test]
fn space_on_an_empty_word_is_ignored() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "  cat");

    assert_eq!(caret(&app), (0, 3));
}

#[test]
fn space_commits_an_unfinished_word() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "c dog");

    assert_eq!(app.words[0].typed, "c");
    assert_eq!(caret(&app), (1, 3));
}

#[test]
fn backspace_deletes_within_a_word() {
    let mut app = app(&["cat"]);
    type_str(&mut app, "ca");
    app.backspace();

    assert_eq!(app.words[0].typed, "c");
    assert_eq!(caret(&app), (0, 1));
}

#[test]
fn backspace_on_an_empty_word_steps_back() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cat ");
    app.backspace();

    // Back inside "cat", with its input intact.
    assert_eq!(caret(&app), (0, 3));
    assert_eq!(app.words[0].typed, "cat");
}

#[test]
fn backspace_at_the_very_start_does_nothing() {
    let mut app = app(&["cat"]);
    app.backspace();

    assert_eq!(caret(&app), (0, 0));
}

#[test]
fn overflow_is_capped() {
    let mut app = app(&["cat"]);
    type_str(&mut app, &"x".repeat(100));

    assert_eq!(app.words[0].typed.chars().count(), 3 + MAX_OVERFLOW);
    // Keystrokes past the cap aren't charged as mistakes either.
    assert_eq!(app.keystrokes(), 3 + MAX_OVERFLOW);
}

// -- phase ------------------------------------------------------------

#[test]
fn the_clock_starts_on_the_first_keystroke() {
    let mut app = app(&["cat"]);
    assert!(!app.is_running());
    assert_eq!(app.elapsed(), Duration::ZERO);

    app.type_char('c');
    assert!(app.is_running());
}

#[test]
fn running_out_of_words_ends_the_test() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cat dog ");

    assert!(app.is_over());
    assert!(!app.is_running());
}

#[test]
fn input_is_ignored_once_the_test_is_over() {
    let mut app = app(&["cat"]);
    type_str(&mut app, "cat ");
    type_str(&mut app, "zzz");
    app.backspace();

    assert_eq!(app.words[0].typed, "cat");
}

#[test]
fn restart_deals_a_fresh_test_but_keeps_quitting() {
    let mut app = app(&["cat"]);
    type_str(&mut app, "cat ");
    app.restart();

    assert!(!app.is_over());
    assert_eq!(caret(&app), (0, 0));
    assert_eq!(app.keystrokes(), 0);
}
