//! Tests for [`super::App`].
//!
//! Split out of `mod.rs` only for length: these are unit tests of a
//! private-by-default type, and reach its private fields through
//! `use super::*` exactly as an inline `mod tests` would.

use super::*;

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
        .worst(crate::misses::WORST)
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

// -- menu -------------------------------------------------------------

#[test]
fn the_menu_opens_on_the_duration_in_force() {
    let mut app = app(&["cat"]);
    app.menu_index = 3;
    app.open_menu();

    assert_eq!(app.screen, Screen::Menu);
    assert_eq!(MENU[app.menu_index], MenuItem::Duration(30));
}

#[test]
fn opening_the_menu_abandons_a_run_in_progress() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cat");
    app.open_menu();

    assert!(!app.is_running());
    assert_eq!(caret(&app), (0, 0));
    assert_eq!(app.records().runs(30), 0);
}

#[test]
fn opening_the_menu_leaves_a_finished_run_alone() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cat dog");
    app.started_at = Some(Instant::now() - Duration::from_secs(5));
    app.type_space();

    let wpm = app.wpm();
    app.open_menu();

    assert!(app.is_over());
    assert_eq!(app.wpm(), wpm);
}

#[test]
fn the_highlight_wraps_at_both_ends() {
    let mut app = app(&["cat"]);
    app.menu_index = 0;

    app.menu_move(-1);
    assert_eq!(app.menu_index, MENU.len() - 1);

    app.menu_move(1);
    assert_eq!(app.menu_index, 0);
}

#[test]
fn choosing_a_duration_restarts_the_test() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cat ");

    app.open_menu();
    app.menu_index = 0; // 15 seconds
    app.menu_select();

    assert_eq!(app.duration(), Duration::from_secs(15));
    assert_eq!(app.screen, Screen::Test);
    assert_eq!(caret(&app), (0, 0));
    // A fresh word list, sized for the new clock.
    assert_eq!(app.words.len(), 15 * WORDS_PER_SECOND);
}

#[test]
fn choosing_a_duration_remembers_it_for_next_time() {
    let mut app = app(&["cat"]);
    app.open_menu();
    app.menu_index = menu_row(MenuItem::Duration(15));
    app.menu_select();

    assert_eq!(app.settings.duration(), Some(15));
}

#[test]
fn a_stored_duration_is_used_at_startup() {
    let mut settings = Settings::detached();
    settings.set_duration(60);
    let app = App::build(
        Records::default(),
        History::default(),
        Banner::detached(),
        settings,
        Themes::detached(),
    );

    assert_eq!(app.duration(), Duration::from_secs(60));
    // The menu opens on it, rather than on the default.
    assert_eq!(MENU[app.duration_index()], MenuItem::Duration(60));
}

#[test]
fn a_duration_the_menu_cant_show_falls_back_to_the_default() {
    // Someone hand-edited settings.tsv to a length with no menu row.
    let mut settings = Settings::detached();
    settings.set_duration(45);
    let app = App::build(
        Records::default(),
        History::default(),
        Banner::detached(),
        settings,
        Themes::detached(),
    );

    assert_eq!(app.duration(), Duration::from_secs(DEFAULT_DURATION));
}

#[test]
fn the_countdown_follows_the_chosen_duration() {
    let mut app = app(&["cat"]);
    app.open_menu();
    app.menu_index = 2; // 60 seconds
    app.menu_select();

    assert_eq!(app.remaining(), Duration::from_secs(60));
}

#[test]
fn records_opens_from_the_menu_and_steps_back_to_it() {
    let mut app = app(&["cat"]);
    app.open_menu();
    app.menu_index = menu_row(MenuItem::Records);
    app.menu_select();
    assert_eq!(app.screen, Screen::Records);

    app.back();
    assert_eq!(app.screen, Screen::Menu);

    app.back();
    assert_eq!(app.screen, Screen::Test);
}

#[test]
fn the_banner_screen_opens_from_the_menu_and_steps_back_to_it() {
    let mut app = app(&["cat"]);
    app.open_menu();
    app.menu_index = menu_row(MenuItem::Banner);
    app.menu_select();
    assert_eq!(app.screen, Screen::Banner);

    app.back();
    assert_eq!(app.screen, Screen::Menu);
}

// -- theme ----------------------------------------------------------------

#[test]
fn the_theme_starts_on_the_default() {
    let app = app(&["cat"]);

    assert_eq!(app.theme().name, crate::theme::DEFAULT);
    assert_eq!(app.theme_index(), 0);
}

#[test]
fn the_banner_can_be_turned_off_and_back_on() {
    let mut app = app(&["cat"]);
    assert!(app.banner_shown());

    app.toggle_banner();
    assert!(!app.banner_shown());
    assert!(app.status().is_some_and(|s| s.contains("off")));

    app.toggle_banner();
    assert!(app.banner_shown());
}

#[test]
fn turning_the_banner_off_is_remembered() {
    let mut app = app(&["cat"]);
    app.toggle_banner();

    assert!(!app.settings.banner_shown());
}

#[test]
fn turning_the_banner_off_leaves_the_art_alone() {
    let mut app = app(&["cat"]);
    let art = app.banner().art().to_string();
    app.toggle_banner();

    // Off is a preference, not a deletion — `x` is what removes art.
    assert_eq!(app.banner().art(), art);
}

#[test]
fn the_banner_colour_comes_from_the_theme() {
    let mut app = app(&["cat"]);
    let first = app.themes().all()[0].banner;
    let second = app.themes().all()[1].banner;

    assert_eq!(app.banner_colour(), first);
    app.theme_move(1);
    assert_eq!(app.banner_colour(), second);
}

#[test]
fn moving_through_the_picker_applies_the_theme() {
    let mut app = app(&["cat"]);
    let second = app.themes().all()[1].name.clone();
    app.theme_move(1);

    assert_eq!(app.theme().name, second);
    assert_eq!(app.theme_index(), 1);
}

#[test]
fn the_picker_wraps_at_both_ends() {
    let mut app = app(&["cat"]);
    let last = app.themes().all().len() - 1;

    app.theme_move(-1);
    assert_eq!(app.theme_index(), last);

    app.theme_move(1);
    assert_eq!(app.theme_index(), 0);
}

#[test]
fn the_theme_picker_opens_from_the_menu_and_steps_back_to_it() {
    let mut app = app(&["cat"]);
    app.open_menu();
    app.menu_index = menu_row(MenuItem::Theme);
    app.menu_select();
    assert_eq!(app.screen, Screen::Theme);

    app.back();
    assert_eq!(app.screen, Screen::Menu);
}

#[test]
fn a_theme_survives_a_restart() {
    let mut app = app(&["cat"]);
    app.theme_move(1);
    let chosen = app.theme().name.clone();
    app.restart();

    assert_eq!(app.theme().name, chosen);
}

#[test]
fn an_edit_request_names_what_to_edit() {
    let mut app = app(&["cat"]);
    assert_eq!(app.take_edit(), None);

    app.request_edit(EditTarget::Themes);
    assert_eq!(app.take_edit(), Some(EditTarget::Themes));
    assert_eq!(app.take_edit(), None);

    app.request_edit(EditTarget::Banner);
    assert_eq!(app.take_edit(), Some(EditTarget::Banner));
}

#[test]
fn a_status_message_belongs_to_the_screen_that_made_it() {
    let mut app = app(&["cat"]);
    app.screen = Screen::Banner;
    app.set_status("something happened");

    app.back();
    assert_eq!(app.status(), None);
}

#[test]
fn leaving_a_screen_never_quits() {
    let mut app = app(&["cat"]);
    app.back();

    assert_eq!(app.screen, Screen::Test);
    assert!(!app.should_quit);
}

#[test]
fn a_restart_keeps_the_chosen_duration() {
    let mut app = app(&["cat"]);
    app.open_menu();
    app.menu_index = 0;
    app.menu_select();
    app.restart();

    assert_eq!(app.duration(), Duration::from_secs(15));
}

// -- records ----------------------------------------------------------

#[test]
fn finishing_a_test_files_a_record() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cat dog");
    // A believable elapsed time, or the run is too fast to score.
    app.started_at = Some(Instant::now() - Duration::from_secs(5));
    app.type_space();

    assert!(app.is_over());
    assert_eq!(app.records().runs(30), 1);
    assert!(app.is_new_best());
}

#[test]
fn a_test_nobody_typed_is_not_a_record() {
    let mut app = app(&["cat"]);
    app.started_at = Some(Instant::now() - app.duration());
    app.tick();

    assert!(app.is_over());
    assert_eq!(app.records().runs(30), 0);
    assert!(!app.is_new_best());
}

#[test]
fn an_abandoned_test_is_not_a_record() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cat ");
    app.restart();

    assert_eq!(app.records().runs(30), 0);
}

// -- history ----------------------------------------------------------

/// The speeds of every run filed at `seconds`, oldest first.
fn history(app: &App, seconds: u64) -> Vec<f64> {
    app.history()
        .at(seconds)
        .iter()
        .map(|run| run.wpm)
        .collect()
}

#[test]
fn a_finished_run_is_filed_in_the_history_too() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cat dog");
    app.started_at = Some(Instant::now() - Duration::from_secs(5));
    app.type_space();

    // The same figure the records were given, not a second reading of a
    // clock that has moved on since.
    assert_eq!(history(&app, 30), vec![app.wpm().unwrap()]);
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
    let runs = history(&app, 30);
    assert_eq!(runs.len(), 2);
    assert!(runs[1] < runs[0]);
}

#[test]
fn a_test_nobody_typed_is_not_in_the_history() {
    let mut app = app(&["cat"]);
    app.started_at = Some(Instant::now() - app.duration());
    app.tick();

    assert!(history(&app, 30).is_empty());
}

#[test]
fn an_abandoned_test_is_not_in_the_history() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cat ");
    app.restart();

    assert!(history(&app, 30).is_empty());
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
    assert_eq!(app.records().runs(30), 2);
}

#[test]
fn remaining_never_goes_negative() {
    let mut app = app(&["cat"]);
    app.started_at = Some(Instant::now() - app.duration() * 2);
    assert_eq!(app.remaining(), Duration::ZERO);
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
