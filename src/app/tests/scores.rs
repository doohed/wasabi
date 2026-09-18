//! What a finished run is filed as, and what isn't filed at all.
//!
//! Fixtures come from [`super`], which is where every test in this module
//! builds its `App` — an in-memory one, so no test can reach the user's real
//! files.

use super::*;

// -- records ----------------------------------------------------------

#[test]
fn finishing_a_test_files_a_record() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cat dog");
    // A believable elapsed time, or the run is too fast to score.
    app.started_at = Some(Instant::now() - Duration::from_secs(5));
    app.type_space();

    assert!(app.is_over());
    assert_eq!(app.records().runs(&key(Mode::default(), 30)), 1);
    assert!(app.is_new_best());
}

#[test]
fn a_test_nobody_typed_is_not_a_record() {
    let mut app = app(&["cat"]);
    app.started_at = Some(Instant::now() - app.duration());
    app.tick();

    assert!(app.is_over());
    assert_eq!(app.records().runs(&key(Mode::default(), 30)), 0);
    assert!(!app.is_new_best());
}

#[test]
fn an_abandoned_test_is_not_a_record() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cat ");
    app.restart();

    assert_eq!(app.records().runs(&key(Mode::default(), 30)), 0);
}

// -- history ----------------------------------------------------------

#[test]
fn a_finished_run_is_filed_in_the_history_too() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cat dog");
    app.started_at = Some(Instant::now() - Duration::from_secs(5));
    app.type_space();

    // The same figure the records were given, not a second reading of a
    // clock that has moved on since.
    assert_eq!(
        history(&app, &key(Mode::default(), 30)),
        vec![app.wpm().unwrap()]
    );
}

#[test]
fn the_history_keeps_the_runs_that_weren_t_bests() {
    let mut app = app(&["cat", "dog"]);

    for seconds in [2, 25] {
        app.restart();
        app.words = ["cat", "dog"].into_iter().map(Word::new).collect();
        type_str(&mut app, "cat dog");
        app.started_at = Some(Instant::now() - Duration::from_secs(seconds));
        app.type_space();
    }

    // The second run was slower, so the record still belongs to the first —
    // and the history has both, which is the whole point of it.
    assert!(!app.is_new_best());
    let runs = history(&app, &key(Mode::default(), 30));
    assert_eq!(runs.len(), 2);
    assert!(runs[1] < runs[0]);
}

#[test]
fn a_test_nobody_typed_is_not_in_the_history() {
    let mut app = app(&["cat"]);
    app.started_at = Some(Instant::now() - app.duration());
    app.tick();

    assert!(history(&app, &key(Mode::default(), 30)).is_empty());
}

#[test]
fn an_abandoned_test_is_not_in_the_history() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cat ");
    app.restart();

    assert!(history(&app, &key(Mode::default(), 30)).is_empty());
}

#[test]
fn a_slower_run_doesnt_clear_the_new_best_flag_of_its_own_run() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cat dog");
    app.started_at = Some(Instant::now() - Duration::from_secs(2));
    app.type_space();
    assert!(app.is_new_best());

    // A second, much slower run over the same words.
    app.restart();
    app.words = ["cat", "dog"].into_iter().map(Word::new).collect();
    type_str(&mut app, "cat dog");
    app.started_at = Some(Instant::now() - Duration::from_secs(25));
    app.type_space();

    assert!(!app.is_new_best());
    assert_eq!(app.records().runs(&key(Mode::default(), 30)), 2);
}

#[test]
fn remaining_never_goes_negative() {
    let mut app = app(&["cat"]);
    app.started_at = Some(Instant::now() - app.duration() * 2);
    assert_eq!(app.remaining(), Duration::ZERO);
}
