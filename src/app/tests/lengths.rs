//! Word tests: chosen like a time, ended by the words rather than the clock.
//!
//! Fixtures come from [`super`], which is where every test in this module
//! builds its `App` — an in-memory one, so no test can reach the user's real
//! files.

use super::*;

/// An app over `targets`, set to a word test as long as they are.
fn word_test(targets: &[&str]) -> App {
    App {
        length: Length::Words(targets.len()),
        ..app(targets)
    }
}

#[test]
fn a_word_test_ends_on_its_last_word_typed_right() {
    let mut app = word_test(&["cat", "dog"]);
    type_str(&mut app, "cat do");
    started(&mut app, 5);
    assert!(!app.is_over());

    // No space: there is nothing after the last word for one to lead into.
    app.type_char('g');

    assert!(app.is_over());
    assert_eq!(
        app.records().runs(&key(Mode::default(), Length::Words(2))),
        1
    );
}

#[test]
fn a_word_test_waits_for_the_last_word_to_be_right() {
    let mut app = word_test(&["cat", "dog"]);
    type_str(&mut app, "cat dgo");

    assert!(!app.is_over());

    // A space still commits it, mistake and all, as in any test.
    started(&mut app, 5);
    app.type_space();
    assert!(app.is_over());
}

#[test]
fn the_last_word_of_a_timed_test_still_wants_its_space() {
    // A timed test is never meant to run out of words; when it does, nothing
    // about it changes.
    let mut app = app(&["cat"]);
    type_str(&mut app, "cat");

    assert!(!app.is_over());
}

#[test]
fn a_word_test_has_no_clock_to_run_out() {
    let mut app = word_test(&["cat", "dog"]);
    type_str(&mut app, "cat");
    started(&mut app, 10_000);
    app.tick();

    assert!(app.is_running());
    assert_eq!(app.remaining(), None);
    assert_eq!(app.limit(), None);
}

#[test]
fn a_word_test_is_scored_over_the_time_it_took() {
    let mut app = word_test(&["cat", "dog"]);
    type_str(&mut app, "cat do");
    started(&mut app, 6);
    app.type_char('g');

    // Seven scoring characters — `cat`, its space, and `dog`, which ended the
    // test without one — over six seconds.
    let wpm = app.wpm().expect("six seconds is plenty");
    assert!((wpm - timeline::wpm(7, 6.0)).abs() < 0.5, "{wpm}");
}

#[test]
fn choosing_a_word_count_deals_exactly_that_many() {
    let mut app = app(&["cat"]);
    toggle(&mut app, MenuItem::Length(Length::Words(25)));

    assert_eq!(app.length(), Length::Words(25));
    assert_eq!(app.words.len(), 25);
    assert_eq!(app.screen, Screen::Test);
    assert_eq!(app.settings.length(), Some(Length::Words(25)));
}

#[test]
fn a_word_count_in_code_mode_deals_that_much_code() {
    let mut app = app(&["cat"]);
    app.set_language(Some(Language::C));
    toggle(&mut app, MenuItem::Length(Length::Words(50)));

    assert_eq!(app.words.len(), 50);
    assert_eq!(app.mode(), Mode::Code(Language::C));
}

#[test]
fn a_stored_word_count_is_used_at_startup() {
    let mut settings = Settings::detached();
    settings.set_length(Length::Words(100));
    let app = App::build(
        Records::default(),
        History::default(),
        Keys::default(),
        Wordlist::detached(),
        Banner::detached(),
        settings,
        Themes::detached(),
    );

    assert_eq!(app.length(), Length::Words(100));
    assert_eq!(app.words.len(), 100);
    assert_eq!(
        MENU[app.length_index()],
        MenuItem::Length(Length::Words(100))
    );
}

#[test]
fn a_word_count_the_menu_cant_show_falls_back_to_the_default() {
    let mut settings = Settings::detached();
    settings.set_length(Length::Words(7));
    let app = App::build(
        Records::default(),
        History::default(),
        Keys::default(),
        Wordlist::detached(),
        Banner::detached(),
        settings,
        Themes::detached(),
    );

    assert_eq!(app.length(), DEFAULT_LENGTH);
}

#[test]
fn a_word_test_and_a_timed_test_keep_separate_records() {
    let mut app = word_test(&["cat"]);
    type_str(&mut app, "ca");
    started(&mut app, 3);
    app.type_char('t');

    assert_eq!(
        app.records().runs(&key(Mode::default(), Length::Words(1))),
        1
    );
    assert_eq!(app.records().runs(&key(Mode::default(), 30)), 0);
}

#[test]
fn every_length_has_a_menu_row() {
    for length in LENGTHS {
        assert!(MENU.contains(&MenuItem::Length(length)), "{length:?}");
    }
}
