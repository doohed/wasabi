use std::time::{Duration, Instant};

use crate::config::banner::Banner;
use crate::config::settings::Settings;
use crate::config::theme::{Theme, Themes};
use crate::scores::history::History;
use crate::scores::records::Records;
use crate::typing::misses::Misses;
use crate::typing::mode::{Language, Mode};
use crate::typing::modifiers::Modifiers;
use crate::typing::snippets;
use crate::typing::timeline::{self, Timeline};
use crate::typing::word::Word;
use crate::typing::wordlist::Wordlist;

/// Test lengths offered in the menu, in seconds.
pub const DURATIONS: [u64; 3] = [15, 30, 60];

/// The length a fresh install starts on.
const DEFAULT_DURATION: u64 = 30;

/// Which screen is in front.
///
/// The results are not a variant: they're `Test` with the clock stopped, which
/// keeps "is the test over" a single question with a single answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Test,
    Menu,
    Words,
    Code,
    Banner,
    Theme,
    Records,
}

/// A file the user can open in `$EDITOR` from inside the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditTarget {
    Words,
    Banner,
    Themes,
}

/// A row of the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    /// Switch the test to this many seconds.
    Duration(u64),
    /// Open the words screen, where the pool the test draws from is chosen.
    Words,
    /// Open the code picker, where a language is chosen — or none of them.
    Code,
    /// Punctuate the words the test deals.
    Punctuation,
    /// Put the occasional number among them.
    Numbers,
    /// Open the banner screen, where the art above the test is chosen.
    Banner,
    /// Open the theme picker.
    Theme,
    /// Open the records table.
    Records,
}

/// The menu, top to bottom.
///
/// What the test is comes first — its length, its words, what is done to them
/// — then how it looks, then how it went.
pub const MENU: [MenuItem; DURATIONS.len() + 7] = [
    MenuItem::Duration(DURATIONS[0]),
    MenuItem::Duration(DURATIONS[1]),
    MenuItem::Duration(DURATIONS[2]),
    MenuItem::Words,
    MenuItem::Code,
    MenuItem::Punctuation,
    MenuItem::Numbers,
    MenuItem::Banner,
    MenuItem::Theme,
    MenuItem::Records,
];

/// The code picker, top to bottom: the way out, then every language.
///
/// Built rather than written out, so adding a language to [`Language::ALL`] is
/// the only edit a new language needs.
pub fn code_rows() -> Vec<Option<Language>> {
    std::iter::once(None)
        .chain(Language::ALL.into_iter().map(Some))
        .collect()
}

/// How many characters past the end of a word the user is allowed to type.
///
/// Without a cap, holding a key would grow the line without bound and wreck
/// the layout.
const MAX_OVERFLOW: usize = 10;

/// How much typing has to have happened before WPM means anything.
///
/// WPM divides by elapsed time, so the first keystroke of a test extrapolates
/// to a six-figure score. There is no *correct* number to show that early —
/// one character is not evidence of a speed — so [`App::wpm`] reports none.
const MIN_ELAPSED: Duration = Duration::from_secs(1);

/// All application state.
///
/// Deliberately free of any ratatui types: the UI reads from `App`, never the
/// other way round. That keeps the typing logic testable without a terminal.
pub struct App {
    /// The full text of the test, one entry per word.
    pub words: Vec<Word>,
    /// Index into `words` of the word currently being typed.
    pub cursor_word: usize,
    pub should_quit: bool,
    /// Which screen is in front.
    pub screen: Screen,
    /// Highlighted row of the menu.
    pub menu_index: usize,

    /// How long this test runs. Chosen from the menu, and survives a restart.
    duration: Duration,
    /// Personal bests, loaded once at startup and written back on every
    /// completed test.
    records: Records,
    /// Every run this install has finished, which is what the records screen
    /// plots. Separate from `records` because a best is one number and a
    /// history is all of them.
    history: History,
    /// Whether the run just finished beat the record for its duration.
    new_best: bool,

    /// The pool the test draws its words from.
    wordlist: Wordlist,

    /// The art drawn above the test.
    banner: Banner,
    /// A one-line message for the screen in front — the result of the last
    /// thing the user asked for. Cleared whenever they leave the screen.
    status: Option<String>,
    /// Set when the user asks to edit one of their files. The event loop picks
    /// this up, because only it can hand the terminal over to `$EDITOR`.
    edit_requested: Option<EditTarget>,
    /// Persisted preferences.
    settings: Settings,
    /// The built-in palettes plus whatever the user's theme file defines.
    themes: Themes,

    /// Set by the first keystroke, not by startup: the clock starts when the
    /// user does, so a test can sit on screen untouched.
    started_at: Option<Instant>,
    /// Set when the timer runs out or the last word is committed. Its presence
    /// *is* the "test is over" flag, and it freezes [`App::elapsed`] so the
    /// results screen doesn't keep counting.
    ended_at: Option<Instant>,

    /// Every character the user has typed, including ones later deleted.
    keystrokes: usize,
    /// How many of those did not match the expected character.
    ///
    /// Counted at keystroke time rather than derived from the final text,
    /// because fixing a typo should still cost you accuracy.
    mistakes: usize,
    /// A reading a second, so the results can show the shape of the run and
    /// not just its total.
    timeline: Timeline,
    /// Which keys the mistakes were made on, so the results can say what to
    /// practise rather than only how it went.
    misses: Misses,
}

impl App {
    /// The app as the user gets it: everything read from disk.
    pub fn new() -> Self {
        Self::build(
            Records::load(),
            History::load(),
            Wordlist::load(),
            Banner::load(),
            Settings::load(),
            Themes::load(),
        )
    }

    /// The app with nothing behind it: every store in memory only.
    ///
    /// For tests. Listed beside [`App::new`] on purpose — adding a store that
    /// touches the disk means editing both, so a test can never quietly
    /// acquire the power to write to the user's real files.
    #[cfg(test)]
    fn detached() -> Self {
        Self::build(
            Records::default(),
            History::default(),
            Wordlist::detached(),
            Banner::detached(),
            Settings::detached(),
            Themes::detached(),
        )
    }

    /// The single constructor both of those go through.
    fn build(
        records: Records,
        history: History,
        wordlist: Wordlist,
        banner: Banner,
        settings: Settings,
        themes: Themes,
    ) -> Self {
        // Only a length the menu can represent: an arbitrary number from a
        // hand-edited file would run a test the menu couldn't show you.
        let seconds = settings
            .duration()
            .filter(|seconds| DURATIONS.contains(seconds))
            .unwrap_or(DEFAULT_DURATION);

        let mut app = Self {
            words: Vec::new(),
            cursor_word: 0,
            should_quit: false,
            screen: Screen::Test,
            menu_index: 0,
            duration: Duration::from_secs(seconds),
            records,
            history,
            new_best: false,
            wordlist,
            banner,
            status: None,
            edit_requested: None,
            settings,
            themes,
            started_at: None,
            ended_at: None,
            keystrokes: 0,
            mistakes: 0,
            timeline: Timeline::default(),
            misses: Misses::default(),
        };

        app.menu_index = app.duration_index();
        app.restart();
        app
    }

    /// Throw the test away and deal a fresh one.
    ///
    /// Everything chosen by the user — the duration, the screen, the records —
    /// survives; only the run itself is reset.
    pub fn restart(&mut self) {
        let seconds = self.duration.as_secs();
        self.words = match self.mode() {
            Mode::Words(modifiers) => self.wordlist.deal(seconds, modifiers),
            Mode::Code(language) => snippets::deal(language, seconds),
        };
        self.cursor_word = 0;
        self.started_at = None;
        self.ended_at = None;
        self.keystrokes = 0;
        self.mistakes = 0;
        self.timeline.clear();
        self.misses.clear();
        self.new_best = false;
    }

    // -- menu -------------------------------------------------------------

    pub fn open_menu(&mut self) {
        // A run in progress is abandoned rather than paused. Pausing would
        // mean a clock that stops and starts, and a score you could game by
        // thinking in the menu; a finished run is left alone, so you can check
        // your records and come back to your results.
        if self.is_running() {
            self.restart();
        }

        // Open on the current setting, not wherever the cursor was left.
        self.menu_index = self.duration_index();
        self.screen = Screen::Menu;
    }

    /// Esc, from wherever it is pressed.
    ///
    /// Records step back to the menu they were opened from; the menu steps
    /// back to the test. Nothing here quits — that is Ctrl-C, so that leaving
    /// a screen can never end the session by accident.
    pub fn back(&mut self) {
        // A message belongs to the screen that produced it.
        self.status = None;

        // The picker borrowed `menu_index`; hand it back pointing at the row
        // that opened it, rather than at whatever it was left on.
        if self.screen == Screen::Code {
            self.menu_index = MENU
                .iter()
                .position(|item| *item == MenuItem::Code)
                .unwrap_or(0);
        }

        self.screen = match self.screen {
            Screen::Words | Screen::Code | Screen::Banner | Screen::Theme | Screen::Records => {
                Screen::Menu
            }
            Screen::Menu | Screen::Test => Screen::Test,
        };
    }

    /// Move the highlight, wrapping at both ends.
    pub fn menu_move(&mut self, delta: isize) {
        let len = MENU.len() as isize;
        self.menu_index = (self.menu_index as isize + delta).rem_euclid(len) as usize;
    }

    /// Move the highlight in the code picker, wrapping at both ends.
    pub fn code_move(&mut self, delta: isize) {
        let len = code_rows().len() as isize;
        self.menu_index = (self.menu_index as isize + delta).rem_euclid(len) as usize;
    }

    /// Type the highlighted language, and go back to the test.
    pub fn code_select(&mut self) {
        let choice = code_rows()[self.menu_index];

        self.set_language(choice);
        self.screen = Screen::Test;
    }

    /// Activate the highlighted row.
    pub fn menu_select(&mut self) {
        match MENU[self.menu_index] {
            MenuItem::Duration(seconds) => {
                self.set_duration(seconds);
                self.screen = Screen::Test;
            }
            MenuItem::Words => self.screen = Screen::Words,
            MenuItem::Code => {
                // Open on the choice in force, not wherever the menu's cursor
                // happened to be — the two lists share `menu_index`.
                let chosen = self.mode().language();
                self.menu_index = code_rows()
                    .iter()
                    .position(|row| *row == chosen)
                    .unwrap_or(0);
                self.screen = Screen::Code;
            }
            MenuItem::Punctuation => self.toggle_punctuation(),
            MenuItem::Numbers => self.toggle_numbers(),
            MenuItem::Banner => self.screen = Screen::Banner,
            MenuItem::Theme => self.screen = Screen::Theme,
            MenuItem::Records => self.screen = Screen::Records,
        }
    }

    /// Whether the setting on this row is in force, for the menu's marker.
    ///
    /// A question per row rather than one highlighted index: the lengths are a
    /// choice of one, and the modifiers are two independent switches, so there
    /// is no single row that is "the" active one.
    pub fn menu_ticked(&self, item: MenuItem) -> bool {
        // The word settings are ticked by what is *in force*, not by what is
        // stored: in code mode they are remembered but have no say, and a tick
        // beside a row that changes nothing would be a lie.
        let modifiers = self.mode().modifiers();

        match item {
            MenuItem::Duration(seconds) => seconds == self.duration.as_secs(),
            MenuItem::Code => self.mode().language().is_some(),
            MenuItem::Punctuation => modifiers.is_some_and(|m| m.punctuation),
            MenuItem::Numbers => modifiers.is_some_and(|m| m.numbers),
            MenuItem::Words | MenuItem::Banner | MenuItem::Theme | MenuItem::Records => false,
        }
    }

    /// Row of `MENU` holding the duration in force, for the menu's marker.
    pub fn duration_index(&self) -> usize {
        MENU.iter()
            .position(|item| *item == MenuItem::Duration(self.duration.as_secs()))
            .unwrap_or(0)
    }

    /// Change the test length, and remember it for next time.
    ///
    /// Always restarts: a half-typed test measured against a different clock
    /// would be meaningless.
    fn set_duration(&mut self, seconds: u64) {
        self.duration = Duration::from_secs(seconds);
        self.settings.set_duration(seconds);
        self.restart();
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// What kind of test this is.
    pub fn mode(&self) -> Mode {
        self.settings.mode()
    }

    /// The name this run's records are filed under: its length, and what kind
    /// of test it was.
    pub fn record_key(&self) -> String {
        self.mode().key(self.duration.as_secs())
    }

    /// Type a language's code, or `None` to go back to the word test.
    ///
    /// The word settings are left on disk untouched, so turning code off finds
    /// punctuation exactly as you left it.
    pub fn set_language(&mut self, language: Option<Language>) {
        self.settings.set_language(language);
        self.restart();
    }

    pub fn toggle_punctuation(&mut self) {
        let mut modifiers = self.settings.modifiers();
        modifiers.punctuation = !modifiers.punctuation;
        self.set_modifiers(modifiers);
    }

    pub fn toggle_numbers(&mut self) {
        let mut modifiers = self.settings.modifiers();
        modifiers.numbers = !modifiers.numbers;
        self.set_modifiers(modifiers);
    }

    /// Change what the test does to its words, and remember it for next time.
    ///
    /// Turns the code test off, because these are settings *of the word test*:
    /// asking for punctuation is asking for the test that can have it, and a
    /// tick appearing beside a row that changed nothing would be worse than
    /// the switch.
    ///
    /// Always restarts, for the same reason a change of length does: the words
    /// on screen were dealt under the old setting, and finishing them would
    /// score a test nobody chose.
    fn set_modifiers(&mut self, modifiers: Modifiers) {
        self.settings.set_modifiers(modifiers);
        self.settings.set_language(None);
        self.restart();
    }

    pub fn records(&self) -> &Records {
        &self.records
    }

    /// Every run this install has finished, for the graph on the records
    /// screen.
    pub fn history(&self) -> &History {
        &self.history
    }

    /// The run just finished beat its record. False for an abandoned test.
    pub fn is_new_best(&self) -> bool {
        self.new_best
    }

    // -- theme ------------------------------------------------------------

    /// The palette everything is drawn in.
    pub fn theme(&self) -> &Theme {
        self.themes.get(self.settings.theme())
    }

    /// Every theme on offer, built-in and user-defined.
    pub fn themes(&self) -> &Themes {
        &self.themes
    }

    /// Row of the picker the current theme sits on.
    pub fn theme_index(&self) -> usize {
        self.themes.index_of(self.settings.theme())
    }

    /// Move through the themes, applying each as it is highlighted.
    ///
    /// Applying on the move rather than on a confirm makes the whole interface
    /// the preview — which is the only honest way to judge a palette — and
    /// leaves nothing to cancel.
    pub fn theme_move(&mut self, delta: isize) {
        let len = self.themes.all().len() as isize;
        let next = (self.theme_index() as isize + delta).rem_euclid(len) as usize;
        let name = self.themes.all()[next].name.clone();

        self.settings.set_theme(&name);
    }

    /// Re-read the theme file, picking up anything just saved to it.
    ///
    /// The selected theme is left alone: if it vanished from the file, `theme()`
    /// falls back on its own, and re-selecting for the user would lose their
    /// choice the moment they made a typo.
    pub fn reload_themes(&mut self) {
        self.themes.reload();

        let custom = self.themes.custom_count();
        let skipped = self.themes.skipped_lines();

        self.status = Some(match (custom, skipped) {
            (0, 0) => "no themes in the file — showing the built-ins".to_string(),
            (_, 0) => format!("loaded {custom} of your themes"),
            (_, 1) => format!("loaded {custom} of your themes — 1 line ignored"),
            (_, _) => format!("loaded {custom} of your themes — {skipped} lines ignored"),
        });
    }

    // -- words ------------------------------------------------------------

    pub fn wordlist(&self) -> &Wordlist {
        &self.wordlist
    }

    /// Re-read the word list, picking up anything just saved to it.
    ///
    /// Restarts, because the words on screen came from the old pool: leaving
    /// them there would make `r` look like it had done nothing.
    pub fn reload_wordlist(&mut self) {
        self.wordlist = Wordlist::load();
        self.restart();

        self.status = Some(if self.wordlist.is_custom() {
            format!("loaded your words — {} of them", self.wordlist.len())
        } else {
            "no words saved — using the built-in list".to_string()
        });
    }

    /// Throw away the user's word list and go back to the built-in.
    pub fn reset_wordlist(&mut self) {
        self.status = Some(match self.wordlist.reset() {
            Ok(()) => "removed your words — using the built-in list".to_string(),
            Err(error) => format!("couldn't remove it: {error}"),
        });

        self.restart();
    }

    // -- banner -----------------------------------------------------------

    pub fn banner(&self) -> &Banner {
        &self.banner
    }

    /// Whether the user wants the art drawn at all.
    ///
    /// Only the preference — whether it *fits* is the layout's business.
    pub fn banner_shown(&self) -> bool {
        self.settings.banner_shown()
    }

    /// Turn the art on or off, and remember which.
    pub fn toggle_banner(&mut self) {
        let shown = !self.banner_shown();
        self.settings.set_banner_shown(shown);

        self.status = Some(if shown {
            "banner on".to_string()
        } else {
            "banner off — the test sits in the middle".to_string()
        });
    }

    /// The banner's colour, which comes from the theme like every other
    /// colour in the interface.
    pub fn banner_colour(&self) -> ratatui::style::Color {
        self.theme().banner
    }

    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status = Some(message.into());
    }

    /// Ask the event loop to open `$EDITOR` on one of the user's files.
    ///
    /// A request rather than an action: `App` has no business knowing there is
    /// a terminal to hand over, let alone how to give it back.
    pub fn request_edit(&mut self, target: EditTarget) {
        self.edit_requested = Some(target);
    }

    /// Consume the request, if there is one.
    pub fn take_edit(&mut self) -> Option<EditTarget> {
        self.edit_requested.take()
    }

    /// Re-read the banner file, picking up anything just saved to it.
    pub fn reload_banner(&mut self) {
        self.banner = Banner::load();

        self.status = Some(if self.banner.is_custom() {
            format!(
                "loaded your art — {} × {}",
                self.banner.width(),
                self.banner.height()
            )
        } else {
            "no art saved — showing the built-in".to_string()
        });
    }

    /// Throw away the user's art and go back to the built-in.
    pub fn reset_banner(&mut self) {
        self.status = Some(match self.banner.reset() {
            Ok(()) => "removed your art — showing the built-in".to_string(),
            Err(error) => format!("couldn't remove it: {error}"),
        });
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    // -- phase ------------------------------------------------------------

    /// The test has ended; the results screen is showing.
    pub fn is_over(&self) -> bool {
        self.ended_at.is_some()
    }

    /// The clock is running. False before the first keystroke and after the end.
    pub fn is_running(&self) -> bool {
        self.started_at.is_some() && !self.is_over()
    }

    /// The word the caret is inside. `None` once every word is committed.
    pub fn current_word(&self) -> Option<&Word> {
        self.words.get(self.cursor_word)
    }

    /// How many characters of the current word have been typed.
    ///
    /// This is the caret's offset *within* the current word, which is what the
    /// renderer needs to draw it. Counted in `char`s, not bytes, so it stays
    /// correct for multi-byte input.
    pub fn cursor_char(&self) -> usize {
        self.current_word()
            .map_or(0, |word| word.typed.chars().count())
    }

    /// Called once per frame, because a test can end with no key pressed.
    ///
    /// The event loop is the only caller; keeping the expiry check here rather
    /// than inside `elapsed()` means a single place decides when time is up.
    pub fn tick(&mut self) {
        if !self.is_running() {
            return;
        }

        // Ended first, so the final reading is taken against a clock that has
        // stopped — otherwise the last second of a test lands on the graph at
        // whatever moment the loop happened to notice.
        if self.expired() {
            self.end_at(self.deadline().expect("a running test has a start"));
        }

        self.sample();
    }

    /// When the clock runs out: exactly one `duration` after the first
    /// keystroke. `None` before the test has started.
    fn deadline(&self) -> Option<Instant> {
        Some(self.started_at? + self.duration)
    }

    /// The clock has run out, whether or not anything has noticed yet.
    ///
    /// The event loop polls on a timer, so between the deadline and the tick
    /// that acts on it there is a window of up to that timer's length. This is
    /// what stops a keystroke landing in that window from counting: it arrived
    /// after time was up, and the loop being busy is not a reason to score it.
    fn expired(&self) -> bool {
        self.deadline()
            .is_some_and(|deadline| Instant::now() >= deadline)
    }

    /// Offer the timeline the run's totals; it decides whether a reading is
    /// due. The locals are for the borrow checker, which can't see that
    /// reading `self` and writing `self.timeline` don't overlap.
    fn sample(&mut self) {
        let at = self.elapsed().as_secs_f64();
        let (correct, standing, mistakes) =
            (self.correct_chars(), self.standing_chars(), self.mistakes);

        self.timeline.tick(at, correct, standing, mistakes);
    }

    /// Stop the clock at `at` and file the result.
    ///
    /// `at` rather than "now", because the two are not the same thing. A test
    /// that runs out of time ends on its deadline, however long the loop took
    /// to notice; only a test that runs out of *words* ends at the moment it
    /// is discovered, because that moment is when it really ended.
    ///
    /// Scoring every 30s run over exactly 30 seconds is what makes two of them
    /// comparable: the alternative divides each by a slightly different
    /// number, and calls the difference a change in your typing.
    ///
    /// The order matters: `ended_at` first, so `wpm()` scores the run over the
    /// time it actually took rather than over a clock still running.
    fn end_at(&mut self, at: Instant) {
        self.ended_at = Some(at);

        // Nothing typed, or over too fast for an honest score: not a result.
        if let Some(wpm) = self.wpm().filter(|_| self.keystrokes > 0) {
            let (key, accuracy) = (self.record_key(), self.accuracy());

            self.new_best = self.records.submit(&key, wpm, accuracy);
            self.history.push(&key, wpm, accuracy);
        }
    }

    // -- clock ------------------------------------------------------------

    /// Time spent typing. Zero before the first keystroke, frozen after the end.
    pub fn elapsed(&self) -> Duration {
        let Some(started_at) = self.started_at else {
            return Duration::ZERO;
        };

        self.ended_at
            .unwrap_or_else(Instant::now)
            .duration_since(started_at)
    }

    /// Time left on the clock, for the countdown in the header.
    pub fn remaining(&self) -> Duration {
        self.duration.saturating_sub(self.elapsed())
    }

    // -- input ------------------------------------------------------------

    /// Append one character to the current word.
    ///
    /// Never advances the word: only a space commits a word (see
    /// [`App::type_space`]). Typing past the end of the target is allowed on
    /// purpose — those characters become `CharState::Extra` — but only up to
    /// `MAX_OVERFLOW`.
    pub fn type_char(&mut self, c: char) {
        if self.is_over() || self.expired() {
            return;
        }
        self.started_at.get_or_insert_with(Instant::now);

        let Some(word) = self.words.get_mut(self.cursor_word) else {
            return; // every word committed: swallow the keystroke
        };

        let position = word.typed.chars().count();
        if position >= word.target.chars().count() + MAX_OVERFLOW {
            return; // overflow cap: don't even count it as a keystroke
        }

        // Anything past the end of the target is wrong by definition.
        let expected = word.target.chars().nth(position);
        self.keystrokes += 1;
        if expected != Some(c) {
            self.mistakes += 1;
        }

        // Overflow is charged to the run but to no key: there was no character
        // to get right, so there is nothing to practise.
        if let Some(expected) = expected {
            self.misses.record(expected, expected == c);
        }

        word.typed.push(c);
    }

    /// Commit the current word and move to the next one.
    ///
    /// A space on an empty word is ignored, so leading and repeated spaces
    /// can't silently skip words. Note that a word is committed as-is: an
    /// unfinished word stays unfinished, and its remaining characters count as
    pub fn type_space(&mut self) {
        if self.is_over() || self.expired() {
            return;
        }

        let typed_something = self
            .current_word()
            .is_some_and(|word| !word.typed.is_empty());

        if typed_something {
            // The space that commits a word is a keystroke like any other.
            // [`App::correct_chars`] credits one per committed word, because
            // the standard WPM definition counts the space inside the five
            // characters of a "word" — so leaving it out here would score
            // `wpm` and `raw_wpm` over different text, and `raw_wpm` would
            // come out *below* the figure it is supposed to bound.
            //
            // Never a mistake: pressing space to end a word is the right key
            // whether or not the word it ended was typed correctly, which is
            // the same call `correct_chars` makes.
            self.keystrokes += 1;
            self.cursor_word += 1;
        }

        // Ran out of words before the clock ran out.
        if self.cursor_word >= self.words.len() {
            self.end_at(Instant::now());
        }
    }

    /// Delete one character, stepping back a word when the current one is empty.
    ///
    /// Stepping back is unconditional here. Only lets you return to
    /// a word you got wrong; if you want that, gate the `else if` on
    /// `!self.words[self.cursor_word - 1].is_correct()`.
    pub fn backspace(&mut self) {
        if self.is_over() {
            return;
        }

        let has_typed = self
            .current_word()
            .is_some_and(|word| !word.typed.is_empty());

        if has_typed {
            self.words[self.cursor_word].typed.pop();
        } else if self.cursor_word > 0 {
            // Land at the end of the previous word's input rather than
            // deleting from it, so one backspace = one visible step.
            self.cursor_word -= 1;
        }
    }

    // -- stats ------------------------------------------------------------

    /// Characters that count towards the score.
    ///
    /// Whole correct words only — see [`Word::scoring_chars`] — plus one per
    /// committed word for the space that followed it, which a word only earns
    /// by being right. The word under the caret is credited for as much of
    /// itself as is still correct, because the clock stops you mid-word.
    pub fn correct_chars(&self) -> usize {
        let committed: usize = self.words[..self.cursor_word]
            .iter()
            .map(|word| match word.scoring_chars(false) {
                0 => 0,
                chars => chars + 1, // and the space that ended it
            })
            .sum();

        committed
            + self
                .current_word()
                .map_or(0, |word| word.scoring_chars(true))
    }

    /// Every character standing in the text, right or wrong, plus the spaces
    /// that were committed.
    ///
    /// What [`App::raw_wpm`] scores. Counted from the text rather than from
    /// [`App::keystrokes`], so typing a word, deleting it and typing it again
    /// is worth one word here rather than two — the fingers moved twice, but
    /// only one word came of it.
    pub fn standing_chars(&self) -> usize {
        let letters: usize = self.words.iter().map(Word::standing_chars).sum();

        letters + self.cursor_word
    }

    /// Words per minute: correct characters / 5, over elapsed minutes.
    ///
    /// `None` for the first `MIN_ELAPSED` of a test, which is both the
    /// divide-by-zero guard and the answer to "what should the first frame
    /// show?" — nothing. Every caller renders that as a placeholder.
    pub fn wpm(&self) -> Option<f64> {
        let elapsed = self.elapsed();
        if elapsed < MIN_ELAPSED {
            return None;
        }

        Some(timeline::wpm(self.correct_chars(), elapsed.as_secs_f64()))
    }

    /// Words per minute counting every keystroke, right or wrong.
    ///
    /// The speed of the fingers where [`App::wpm`] is the speed of the typing:
    /// the gap between the two is what the mistakes cost. Never below
    /// [`App::wpm`] — both count the same characters, and this one stops
    /// short of asking whether they were the right ones.
    pub fn raw_wpm(&self) -> Option<f64> {
        let elapsed = self.elapsed();
        if elapsed < MIN_ELAPSED {
            return None;
        }

        Some(timeline::wpm(self.standing_chars(), elapsed.as_secs_f64()))
    }

    /// The run second by second, for the graph on the results screen.
    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    /// Share of keystrokes that hit the right character, as a percentage.
    pub fn accuracy(&self) -> f64 {
        if self.keystrokes == 0 {
            return 100.0;
        }

        (self.keystrokes - self.mistakes) as f64 / self.keystrokes as f64 * 100.0
    }

    pub fn keystrokes(&self) -> usize {
        self.keystrokes
    }

    pub fn mistakes(&self) -> usize {
        self.mistakes
    }

    /// Where the mistakes landed, for the worst keys on the results screen.
    pub fn misses(&self) -> &Misses {
        &self.misses
    }
}

#[cfg(test)]
mod tests;
