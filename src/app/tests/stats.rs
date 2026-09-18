//! The figures a run produces, and when they mean anything.
//!
//! Fixtures come from [`super`], which is where every test in this module
//! builds its `App` — an in-memory one, so no test can reach the user's real
//! files.

use super::*;

// -- stats ------------------------------------------------------------

#[test]
fn correct_chars_counts_committed_spaces() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cat do");

    // c-a-t + the space + d-o
    assert_eq!(app.correct_chars(), 6);
}

#[test]
fn the_space_that_commits_a_word_is_a_keystroke() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cat do");

    // The same six characters `correct_chars` counts. Counting the space on
    // one side and not the other scored the two speeds over different text,
    // and put `raw_wpm` below the figure it is supposed to bound.
    assert_eq!(app.keystrokes(), 6);
    assert_eq!(app.mistakes(), 0);
    assert_eq!(app.accuracy(), 100.0);
}

#[test]
fn the_space_after_a_botched_word_is_still_the_right_key() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cxt ");

    // One bad letter, and a space that was exactly the key to press — the
    // word's own errors are charged against the word, not against the space.
    assert_eq!(app.keystrokes(), 4);
    assert_eq!(app.mistakes(), 1);
}

#[test]
fn a_space_that_commits_nothing_costs_nothing() {
    let mut app = app(&["cat"]);
    type_str(&mut app, "  ");

    assert_eq!(app.keystrokes(), 0);
    assert_eq!(app.accuracy(), 100.0);
}

#[test]
fn raw_wpm_never_falls_below_wpm() {
    let mut app = app(&["cat", "dog", "emu"]);
    type_str(&mut app, "cat dgo emu");

    let now = Instant::now();
    app.started_at = Some(now - Duration::from_secs(10));
    app.ended_at = Some(now);

    let wpm = app.wpm().expect("ten seconds is plenty");
    let raw = app.raw_wpm().expect("likewise");

    // Both count the same characters; raw just stops short of asking whether
    // they were the right ones, so a run with mistakes has to leave a gap.
    assert!(raw > wpm, "raw {raw} should clear wpm {wpm}");
}

#[test]
fn a_perfect_run_scores_the_same_either_way() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cat dog");

    let now = Instant::now();
    app.started_at = Some(now - Duration::from_secs(10));
    app.ended_at = Some(now);

    // Nothing was thrown away, so there is nothing for raw to add.
    assert_eq!(app.wpm(), app.raw_wpm());
}

#[test]
fn wrong_characters_dont_score() {
    let mut app = app(&["cat"]);
    type_str(&mut app, "cxt");

    assert_eq!(app.correct_chars(), 2);
}

#[test]
fn extra_characters_dont_score() {
    let mut app = app(&["cat"]);
    type_str(&mut app, "catzz");

    assert_eq!(app.correct_chars(), 3);
}

#[test]
fn accuracy_is_charged_at_keystroke_time() {
    let mut app = app(&["cat"]);
    type_str(&mut app, "cx");
    app.backspace();
    type_str(&mut app, "at");

    // The text is now perfect, but the typo still counts: 4 keystrokes, 1 bad.
    assert_eq!(app.words[0].typed, "cat");
    assert_eq!(app.keystrokes(), 4);
    assert_eq!(app.mistakes(), 1);
    assert_eq!(app.accuracy(), 75.0);
}

#[test]
fn accuracy_starts_at_a_hundred() {
    let app = app(&["cat"]);
    assert_eq!(app.accuracy(), 100.0);
}

/// The characters `app` got wrong most, worst first.
fn worst_keys(app: &App) -> Vec<char> {
    app.misses()
        .worst(crate::typing::misses::WORST)
        .into_iter()
        .map(|miss| miss.key)
        .collect()
}

#[test]
fn a_miss_is_charged_to_the_key_you_meant_to_hit() {
    let mut app = app(&["cat"]);
    type_str(&mut app, "cxt");

    // `x` is only where the finger landed; `a` is the key to practise.
    assert_eq!(worst_keys(&app), vec!['a']);
}

#[test]
fn a_typo_fixed_still_names_its_key() {
    let mut app = app(&["cat"]);
    type_str(&mut app, "cx");
    app.backspace();
    type_str(&mut app, "at");

    // Same rule as accuracy: the mistake happened, and the text being right
    // afterwards doesn't unmake it.
    assert_eq!(worst_keys(&app), vec!['a']);
}

#[test]
fn overflow_costs_accuracy_but_names_no_key() {
    let mut app = app(&["cat"]);
    type_str(&mut app, "catzz");

    // There was no character to get right, so there is nothing to practise.
    assert_eq!(app.mistakes(), 2);
    assert!(worst_keys(&app).is_empty());
}

#[test]
fn a_restart_forgets_the_worst_keys() {
    let mut app = app(&["cat"]);
    type_str(&mut app, "cxt");
    app.restart();

    assert!(worst_keys(&app).is_empty());
}

#[test]
fn there_is_no_wpm_before_the_clock_starts() {
    let app = app(&["cat"]);
    assert_eq!(app.wpm(), None);
}

#[test]
fn there_is_no_wpm_for_a_sub_second_test() {
    // A burst of correct characters in no time at all would otherwise
    // extrapolate to a six-figure score.
    let mut app = app(&["cat"]);
    type_str(&mut app, "cat");
    app.ended_at = Some(Instant::now());

    assert_eq!(app.wpm(), None);
}

#[test]
fn wpm_is_correct_chars_over_five_per_minute() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cat dog");

    // Freeze a known 30s elapsed, so the arithmetic is checkable:
    // 7 correct chars / 5 = 1.4 words in half a minute = 2.8 wpm.
    let now = Instant::now();
    app.started_at = Some(now - Duration::from_secs(30));
    app.ended_at = Some(now);

    let wpm = app.wpm().expect("30s is plenty of elapsed time");
    assert!((wpm - 2.8).abs() < 1e-9, "got {wpm}");
}

#[test]
fn the_clock_freezes_when_the_test_ends() {
    let mut app = app(&["cat"]);
    let now = Instant::now();
    app.started_at = Some(now - Duration::from_secs(10));
    app.ended_at = Some(now);

    let first = app.elapsed();
    let second = app.elapsed();
    assert_eq!(first, second);
    assert_eq!(first.as_secs(), 10);
}

#[test]
fn tick_ends_the_test_when_time_runs_out() {
    let mut app = app(&["cat"]);
    app.type_char('c');
    app.tick();
    assert!(!app.is_over());

    app.started_at = Some(Instant::now() - app.duration());
    app.tick();
    assert!(app.is_over());
}

// -- the shape of a run -----------------------------------------------

#[test]
fn the_run_is_read_as_it_goes_on() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cat ");

    // Frames a comfortable distance apart. Consecutive seconds would have
    // this racing a clock the test can't stop: every `started_at` is set
    // against a fresh `now`, so two readings a nominal second apart can land
    // a hair under one.
    for second in [1, 3, 5] {
        app.started_at = Some(Instant::now() - Duration::from_secs(second));
        app.tick();
    }

    assert_eq!(app.timeline().samples().len(), 3);
}

#[test]
fn frames_inside_the_same_second_dont_each_get_a_reading() {
    let mut app = app(&["cat"]);
    app.type_char('c');

    for _ in 0..10 {
        app.tick();
    }

    assert!(app.timeline().samples().is_empty());
}

#[test]
fn the_second_the_clock_runs_out_in_still_gets_read() {
    let mut app = app(&["cat"]);
    app.type_char('c');
    app.started_at = Some(Instant::now() - app.duration());
    app.tick();

    assert!(app.is_over());
    assert!(
        !app.timeline().samples().is_empty(),
        "the reading has to be taken before the test is ended, or it's lost"
    );
}

#[test]
fn a_restart_forgets_the_shape_of_the_last_run() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cat ");
    app.started_at = Some(Instant::now() - Duration::from_secs(2));
    app.tick();
    assert!(!app.timeline().samples().is_empty());

    app.restart();
    assert!(app.timeline().samples().is_empty());
}

#[test]
fn raw_wpm_counts_the_keystrokes_that_wpm_throws_away() {
    let mut app = app(&["cat"]);
    type_str(&mut app, "cxt");

    // A round minute, so the arithmetic is checkable: 2 correct characters
    // of 3 typed, over one minute.
    let now = Instant::now();
    app.started_at = Some(now - Duration::from_secs(60));
    app.ended_at = Some(now);

    let wpm = app.wpm().expect("a minute is plenty of elapsed time");
    let raw = app.raw_wpm().expect("likewise");

    assert!((wpm - 0.4).abs() < 1e-9, "got {wpm}");
    assert!((raw - 0.6).abs() < 1e-9, "got {raw}");
}

#[test]
fn there_is_no_raw_wpm_before_the_clock_starts() {
    let app = app(&["cat"]);
    assert_eq!(app.raw_wpm(), None);
}
