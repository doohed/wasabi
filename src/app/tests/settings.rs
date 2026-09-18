//! The menu, and everything it changes.
//!
//! Fixtures come from [`super`], which is where every test in this module
//! builds its `App` — an in-memory one, so no test can reach the user's real
//! files.

use super::*;

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
    assert_eq!(app.records().runs(&key(Mode::default(), 30)), 0);
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
    assert_eq!(
        app.words.len(),
        15 * crate::typing::wordlist::WORDS_PER_SECOND
    );
}

#[test]
fn choosing_a_duration_remembers_it_for_next_time() {
    let mut app = app(&["cat"]);
    app.open_menu();
    app.menu_index = menu_row(MenuItem::Duration(15));
    app.menu_select();

    assert_eq!(app.settings.duration(), Some(15));
}

// -- modifiers --------------------------------------------------------

/// Turn a modifier on or off through the menu, as a user would.
fn toggle(app: &mut App, item: MenuItem) {
    app.open_menu();
    app.menu_index = menu_row(item);
    app.menu_select();
}

#[test]
fn a_modifier_toggles_from_the_menu_and_is_remembered() {
    let mut app = app(&["cat"]);
    assert_eq!(app.mode(), Mode::default());

    toggle(&mut app, MenuItem::Punctuation);
    assert!(app.mode().modifiers().unwrap().punctuation);
    assert!(!app.mode().modifiers().unwrap().numbers);
    assert!(app.settings.modifiers().punctuation);

    toggle(&mut app, MenuItem::Punctuation);
    assert!(!app.mode().modifiers().unwrap().punctuation);
}

#[test]
fn toggling_a_modifier_stays_in_the_menu() {
    let mut app = app(&["cat"]);
    toggle(&mut app, MenuItem::Numbers);

    // Unlike a duration, which closes the menu: the two modifiers are
    // switches, and you may well want to flip both.
    assert_eq!(app.screen, Screen::Menu);
    assert!(app.mode().modifiers().unwrap().numbers);
}

#[test]
fn toggling_a_modifier_deals_a_fresh_test() {
    let mut app = app(&["cat", "dog"]);
    type_str(&mut app, "cat ");
    toggle(&mut app, MenuItem::Punctuation);

    // The words on screen were dealt under the old setting, so finishing them
    // would score a test nobody chose.
    assert_eq!(caret(&app), (0, 0));
    assert!(!app.is_running());
}

#[test]
fn the_menu_ticks_every_setting_in_force() {
    let mut app = app(&["cat"]);
    toggle(&mut app, MenuItem::Punctuation);

    assert!(app.menu_ticked(MenuItem::Duration(30)));
    assert!(!app.menu_ticked(MenuItem::Duration(15)));
    assert!(app.menu_ticked(MenuItem::Punctuation));
    assert!(!app.menu_ticked(MenuItem::Numbers));
    // Rows that open a screen aren't settings, so nothing is in force.
    assert!(!app.menu_ticked(MenuItem::Words));
}

#[test]
fn a_modified_run_is_filed_under_its_own_name() {
    let mut app = app(&["cat", "dog"]);
    toggle(&mut app, MenuItem::Punctuation);
    // Its own row, not the plain test's.
    assert_ne!(app.record_key(), key(Mode::default(), 30));

    app.words = ["cat", "dog"].into_iter().map(Word::new).collect();
    app.screen = Screen::Test;
    type_str(&mut app, "cat dog");
    app.started_at = Some(Instant::now() - Duration::from_secs(5));
    app.type_space();

    // A punctuated test is a different test, so the plain 30s record is
    // untouched and doesn't have to compete with it.
    assert_eq!(app.records().runs(&app.record_key()), 1);
    assert_eq!(app.records().runs(&key(Mode::default(), 30)), 0);
    assert_eq!(history(&app, &app.record_key()).len(), 1);
    assert!(history(&app, &key(Mode::default(), 30)).is_empty());
}

// -- words ------------------------------------------------------------

#[test]
fn the_words_screen_opens_from_the_menu_and_steps_back_to_it() {
    let mut app = app(&["cat"]);
    app.open_menu();
    app.menu_index = menu_row(MenuItem::Words);
    app.menu_select();

    assert_eq!(app.screen, Screen::Words);

    app.back();
    assert_eq!(app.screen, Screen::Menu);
}

#[test]
fn a_fresh_install_types_the_built_in_words() {
    let app = app(&["cat"]);

    assert!(!app.wordlist().is_custom());
    assert_eq!(app.wordlist().len(), 200);
}

#[test]
fn a_stored_duration_is_used_at_startup() {
    let mut settings = Settings::detached();
    settings.set_duration(60);
    let app = App::build(
        Records::default(),
        History::default(),
        Wordlist::detached(),
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
        Wordlist::detached(),
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

    assert_eq!(app.theme().name, crate::config::theme::DEFAULT);
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

// -- code -------------------------------------------------------------

/// Pick a row of the code picker, as a user would.
fn pick_code(app: &mut App, choice: Option<Language>) {
    app.open_menu();
    app.menu_index = menu_row(MenuItem::Code);
    app.menu_select();

    assert_eq!(app.screen, Screen::Code);
    app.menu_index = code_rows().iter().position(|row| *row == choice).unwrap();
    app.code_select();
}

#[test]
fn choosing_a_language_deals_code() {
    let mut app = app(&["cat"]);
    pick_code(&mut app, Some(Language::Rust));

    assert_eq!(app.mode(), Mode::Code(Language::Rust));
    assert_eq!(app.screen, Screen::Test);
    // Dealt from the snippets, not the word pool: the first word opens a line.
    assert_eq!(app.words[0].indent, Some(0));
    assert!(app.words.iter().any(|word| word.indent == Some(4)));
}

#[test]
fn the_picker_opens_on_the_language_in_force() {
    let mut app = app(&["cat"]);
    pick_code(&mut app, Some(Language::C));

    app.open_menu();
    app.menu_index = menu_row(MenuItem::Code);
    app.menu_select();

    assert_eq!(code_rows()[app.menu_index], Some(Language::C));
}

#[test]
fn leaving_the_picker_hands_the_menu_cursor_back() {
    let mut app = app(&["cat"]);
    app.open_menu();
    app.menu_index = menu_row(MenuItem::Code);
    app.menu_select();

    // The picker borrowed `menu_index`; stepping back must not leave the menu
    // highlighting whatever row that index happens to name.
    app.code_move(1);
    app.back();

    assert_eq!(app.screen, Screen::Menu);
    assert_eq!(MENU[app.menu_index], MenuItem::Code);
}

#[test]
fn turning_code_off_finds_the_word_settings_as_they_were() {
    let mut app = app(&["cat"]);
    toggle(&mut app, MenuItem::Punctuation);

    pick_code(&mut app, Some(Language::C));
    // Remembered, but with no say over a snippet — so no tick beside the row.
    assert_eq!(app.mode().modifiers(), None);
    assert!(!app.menu_ticked(MenuItem::Punctuation));
    assert!(app.menu_ticked(MenuItem::Code));

    pick_code(&mut app, None);
    assert!(app.mode().modifiers().unwrap().punctuation);
    assert!(app.menu_ticked(MenuItem::Punctuation));
}

#[test]
fn asking_for_punctuation_asks_for_the_test_that_can_have_it() {
    let mut app = app(&["cat"]);
    pick_code(&mut app, Some(Language::Rust));

    toggle(&mut app, MenuItem::Numbers);

    // A tick appearing beside a row that changed nothing would be worse than
    // the switch back to words.
    assert_eq!(
        app.mode(),
        Mode::Words(Modifiers {
            punctuation: false,
            numbers: true
        })
    );
    assert!(!app.menu_ticked(MenuItem::Code));
}

#[test]
fn a_code_run_is_filed_under_its_language() {
    let mut app = app(&["cat", "dog"]);
    pick_code(&mut app, Some(Language::C));
    // Its own row, not the plain test's.
    assert_ne!(app.record_key(), key(Mode::default(), 30));

    app.words = ["cat", "dog"].into_iter().map(Word::new).collect();
    type_str(&mut app, "cat dog");
    app.started_at = Some(Instant::now() - Duration::from_secs(5));
    app.type_space();

    // A snippet full of braces scores nothing like a page of common words, so
    // the plain 30s record is untouched.
    assert_eq!(app.records().runs(&app.record_key()), 1);
    assert_eq!(app.records().runs(&key(Mode::default(), 30)), 0);
    assert_eq!(history(&app, &app.record_key()).len(), 1);
}
