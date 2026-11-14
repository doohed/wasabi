//! The pace caret: where a run at your personal best would be by now.
//!
//! Fixtures come from [`super`], which is where every test in this module
//! builds its `App` — an in-memory one, so no test can reach the user's real
//! files.

use super::*;

/// A running test at the default setting, with a best of `wpm` to race, the
/// pace caret on, and the clock `seconds` in.
fn racing(wpm: f64, seconds: u64) -> App {
    let mut app = app(&["cat", "dog"]);
    let key = app.record_key();
    app.records.submit(&key, wpm, 100.0);
    app.settings.set_pace_shown(true);

    type_str(&mut app, "c");
    started(&mut app, seconds);
    app
}

#[test]
fn the_pace_caret_runs_at_your_best() {
    // 60 wpm is five characters a second.
    let app = racing(60.0, 10);
    assert_eq!(app.pace(), Some(50));
}

#[test]
fn the_pace_caret_is_off_until_turned_on() {
    let mut app = racing(60.0, 10);
    app.settings.set_pace_shown(false);

    assert_eq!(app.pace(), None);
}

#[test]
fn there_is_no_pace_without_a_best_to_race() {
    let mut app = app(&["cat"]);
    app.settings.set_pace_shown(true);
    type_str(&mut app, "c");

    assert_eq!(app.pace(), None);
}

#[test]
fn the_pace_caret_waits_for_the_first_keystroke() {
    let mut app = racing(60.0, 10);
    app.restart();

    assert_eq!(app.pace(), None);
}

#[test]
fn the_pace_caret_races_the_best_at_this_setting() {
    let mut app = racing(60.0, 10);
    // A much faster best at another length is not this test's best.
    app.records.submit(&key(Mode::default(), 15), 200.0, 100.0);

    assert_eq!(app.pace(), Some(50));
}

#[test]
fn the_pace_caret_toggles_from_the_menu_and_is_remembered() {
    let mut app = app(&["cat"]);
    toggle(&mut app, MenuItem::Pace);

    assert!(app.pace_shown());
    assert!(app.menu_ticked(MenuItem::Pace));
    assert!(app.settings.pace_shown());
    // A switch, like the modifiers: the menu stays open.
    assert_eq!(app.screen, Screen::Menu);

    toggle(&mut app, MenuItem::Pace);
    assert!(!app.pace_shown());
}
