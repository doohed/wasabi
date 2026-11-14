//! Practice: the keys you keep missing, dealt back to you.
//!
//! Fixtures come from [`super`], which is where every test in this module
//! builds its `App` — an in-memory one, so no test can reach the user's real
//! files.

use super::*;

/// A run's misses: `key` asked for `attempts` times, missed `missed` of them.
fn misses(entries: &[(char, usize, usize)]) -> Misses {
    let mut misses = Misses::default();
    for &(key, attempts, missed) in entries {
        for index in 0..attempts {
            misses.record(key, index >= missed);
        }
    }
    misses
}

#[test]
fn practice_is_off_until_turned_on() {
    let mut app = app(&["cat"]);
    app.keys.absorb(&misses(&[('v', 20, 5)]));
    app.restart();

    assert!(app.focus().is_empty());
    assert!(!app.menu_ticked(MenuItem::Practice));
}

#[test]
fn practice_drills_the_weakest_keys() {
    let mut app = app(&["cat"]);
    app.keys
        .absorb(&misses(&[('v', 20, 2), ('w', 20, 8), ('e', 100, 1)]));
    toggle(&mut app, MenuItem::Practice);

    // Weakest first, by rate: `w` missed two in five, `v` one in ten.
    assert_eq!(app.focus()[..2], ['w', 'v']);
    assert!(app.menu_ticked(MenuItem::Practice));
    assert!(app.settings.modifiers().practice);
}

#[test]
fn practice_never_names_more_keys_than_it_can_drill() {
    let mut app = app(&["cat"]);
    let weak: Vec<(char, usize, usize)> = "bcdfghjklmnp".chars().map(|key| (key, 20, 5)).collect();
    app.keys.absorb(&misses(&weak));
    toggle(&mut app, MenuItem::Practice);

    assert_eq!(app.focus().len(), FOCUS);
}

#[test]
fn a_key_no_word_holds_is_not_practised() {
    let mut app = app(&["cat"]);
    // Digits come from the numbers setting, never from the pool.
    app.keys.absorb(&misses(&[('7', 20, 15), ('v', 20, 2)]));
    toggle(&mut app, MenuItem::Practice);

    assert_eq!(app.focus(), ['v']);
}

#[test]
fn practice_with_nothing_to_practise_is_an_ordinary_test() {
    let mut app = app(&["cat"]);
    toggle(&mut app, MenuItem::Practice);

    assert!(app.focus().is_empty());
    assert_eq!(app.words.len(), app.length().count());
}

#[test]
fn a_practice_run_keeps_its_own_record() {
    let mut app = app(&["cat"]);
    toggle(&mut app, MenuItem::Practice);

    assert!(app.record_key().ends_with("+k"), "{}", app.record_key());
    assert_ne!(app.record_key(), key(Mode::default(), 30));
}

#[test]
fn turning_practice_on_turns_code_off() {
    let mut app = app(&["cat"]);
    app.set_language(Some(Language::Rust));
    toggle(&mut app, MenuItem::Practice);

    // Like the other word settings: asking for it is asking for the test
    // that can have it.
    assert_eq!(app.mode().language(), None);
    assert!(app.menu_ticked(MenuItem::Practice));
}

#[test]
fn a_finished_run_teaches_the_tally() {
    let mut app = app(&["vvvvv", "vvvvv", "end"]);
    type_str(&mut app, "vbvvv vvvvv end");
    started(&mut app, 5);
    app.type_space();

    assert!(app.is_over());
    assert_eq!(app.keys.weakest(), ['v']);
}

#[test]
fn an_abandoned_run_teaches_it_nothing() {
    let mut app = app(&["vvvvv", "vvvvv", "end"]);
    type_str(&mut app, "vbvvv vbvvv ");
    app.restart();

    assert!(app.keys.weakest().is_empty());
}
