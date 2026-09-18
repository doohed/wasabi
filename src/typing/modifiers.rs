//! What the test does to the words after they're drawn.
//!
//! Punctuation and numbers are transforms over a dealt list rather than word
//! lists of their own, which is what lets them apply to *your* pool: turn
//! punctuation on over a Spanish list and you get punctuated Spanish, with
//! nothing shipped to make that work.

use rand::RngExt;

/// Roughly one word in this many gets punctuation attached.
const PUNCTUATED: u32 = 4;
/// Roughly one word in this many is replaced by a number.
const NUMBERED: u32 = 8;

/// Marks that end a sentence, so the next word is capitalised. Commonest
/// first, which is what [`mark`] weights on.
const SENTENCE: [char; 3] = ['.', '?', '!'];
/// Marks that don't.
const CLAUSE: [char; 3] = [',', ';', ':'];

/// How much more often the first mark of a set is picked than either of the
/// others.
///
/// Prose is overwhelmingly full stops and commas. Picking evenly gives a test
/// where every third sentence ends in an exclamation mark and colons turn up
/// as often as commas, which reads as a tour of the punctuation rather than as
/// writing.
const COMMON: u32 = 6;

/// What the test is doing to its words, beyond dealing them.
///
/// `Copy` and tiny, because it is passed to everything that deals or files a
/// run rather than reached for through the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub punctuation: bool,
    pub numbers: bool,
}

impl Modifiers {
    /// How a record at this setting is named, appended to the test length.
    ///
    /// A punctuated test is a different test — the same argument the duration
    /// makes — so it keeps its own personal best rather than competing with
    /// the plain one and losing. Reached through
    /// [`crate::typing::mode::Mode::key`], which is what knows whether the
    /// word test is the one being typed at all.
    pub(super) fn key(self, seconds: u64) -> String {
        let mut key = seconds.to_string();

        if self.punctuation {
            key.push_str("+p");
        }
        if self.numbers {
            key.push_str("+n");
        }

        key
    }

    /// What to call this setting on screen.
    pub fn label(self) -> Option<String> {
        match (self.punctuation, self.numbers) {
            (false, false) => None,
            (true, false) => Some("punctuation".to_string()),
            (false, true) => Some("numbers".to_string()),
            (true, true) => Some("punctuation + numbers".to_string()),
        }
    }

    /// Rewrite a dealt list in place.
    ///
    /// Numbers first, so a number can pick up a comma like any other word and
    /// the capitalising pass sees the list it will actually be typed as.
    pub fn apply(self, words: &mut [String], rng: &mut impl RngExt) {
        if self.numbers {
            numerify(words, rng);
        }
        if self.punctuation {
            punctuate(words, rng);
        }
    }
}

/// Replace the occasional word with a number.
///
/// Never the first word: a test that opens on a bare number reads as a
/// rendering fault rather than a setting you turned on.
fn numerify(words: &mut [String], rng: &mut impl RngExt) {
    for word in words.iter_mut().skip(1) {
        if rng.random_ratio(1, NUMBERED) {
            // Up to four digits. Long enough to have to look, short enough not
            // to become the whole test.
            *word = rng.random_range(0..10_000u32).to_string();
        }
    }
}

/// Attach punctuation to the occasional word, and capitalise what follows a
/// full stop.
///
/// The capitalising is a second pass rather than part of the first, because
/// whether a word starts a sentence is a fact about the word *before* it —
/// deciding it on the way past would mean guessing.
fn punctuate(words: &mut [String], rng: &mut impl RngExt) {
    for word in words.iter_mut() {
        if !rng.random_ratio(1, PUNCTUATED) {
            continue;
        }

        // Weighted by hand rather than uniformly: a test where every twentieth
        // word is in brackets is a bracket drill.
        match rng.random_range(0..20u32) {
            0..=7 => word.push(mark(&SENTENCE, rng)),
            8..=17 => word.push(mark(&CLAUSE, rng)),
            18 => *word = format!("\"{word}\""),
            _ => *word = format!("({word})"),
        }
    }

    capitalise_sentences(words);
}

/// One mark from `set`, the first far more often than the rest.
fn mark(set: &[char; 3], rng: &mut impl RngExt) -> char {
    if rng.random_ratio(COMMON, COMMON + 2) {
        set[0]
    } else {
        set[1 + rng.random_range(0..2usize)]
    }
}

/// Capitalise the first word, and every word after a sentence-ending mark.
fn capitalise_sentences(words: &mut [String]) {
    let mut starting = true;

    for word in words.iter_mut() {
        if starting {
            capitalise(word);
        }

        // Looks past a closing quote or bracket, so `end."` still ends the
        // sentence it was wrapped around.
        starting = word
            .chars()
            .rev()
            .find(|c| !matches!(c, '"' | ')'))
            .is_some_and(|c| SENTENCE.contains(&c));
    }
}

/// Upper-case the first letter of `word`, leaving anything else alone.
///
/// Finds the first letter rather than the first character, so a word already
/// wrapped in quotes is capitalised inside them and a number is untouched.
fn capitalise(word: &mut String) {
    let Some((at, letter)) = word.char_indices().find(|(_, c)| c.is_alphabetic()) else {
        return; // a number, or punctuation on its own
    };

    let upper: String = letter.to_uppercase().collect();
    word.replace_range(at..at + letter.len_utf8(), &upper);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    /// A fixed generator, so a test about randomness has an answer.
    fn rng() -> impl RngExt {
        rand::rngs::SmallRng::seed_from_u64(42)
    }

    fn words(text: &str) -> Vec<String> {
        text.split_whitespace().map(str::to_string).collect()
    }

    const PLAIN: Modifiers = Modifiers {
        punctuation: false,
        numbers: false,
    };
    const BOTH: Modifiers = Modifiers {
        punctuation: true,
        numbers: true,
    };

    #[test]
    fn nothing_turned_on_changes_nothing() {
        let mut list = words("the cat sat on the mat");
        let before = list.clone();

        PLAIN.apply(&mut list, &mut rng());

        assert_eq!(list, before);
    }

    #[test]
    fn a_record_at_the_plain_setting_keeps_the_bare_number() {
        // Which is what every records file written before this existed holds.
        assert_eq!(PLAIN.key(30), "30");
    }

    #[test]
    fn each_setting_keeps_its_own_record() {
        let punctuation = Modifiers {
            punctuation: true,
            numbers: false,
        };
        let numbers = Modifiers {
            punctuation: false,
            numbers: true,
        };

        assert_eq!(punctuation.key(30), "30+p");
        assert_eq!(numbers.key(30), "30+n");
        assert_eq!(BOTH.key(30), "30+p+n");
        assert_eq!(BOTH.key(15), "15+p+n");
    }

    #[test]
    fn only_a_setting_that_is_on_has_a_label() {
        assert_eq!(PLAIN.label(), None);
        assert_eq!(BOTH.label().unwrap(), "punctuation + numbers");
    }

    #[test]
    fn punctuating_leaves_the_words_where_they_were() {
        let mut list = words("the cat sat on the mat and looked around");
        let before = list.clone();

        punctuate(&mut list, &mut rng());

        // Every word is still its old self with marks attached — the list is
        // the test, and reordering it would make the setting a different one.
        assert_eq!(list.len(), before.len());
        for (after, before) in list.iter().zip(&before) {
            let stripped: String = after
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase();
            assert_eq!(stripped, *before, "{after} is not {before}");
        }
    }

    #[test]
    fn the_common_mark_is_the_one_you_mostly_get() {
        let mut counts = [0; 3];
        let mut rng = rng();

        for _ in 0..1000 {
            let got = mark(&SENTENCE, &mut rng);
            counts[SENTENCE.iter().position(|c| *c == got).unwrap()] += 1;
        }

        // Full stops far ahead, and the other two still turning up.
        assert!(counts[0] > counts[1] * 3, "{counts:?}");
        assert!(counts[0] > counts[2] * 3, "{counts:?}");
        assert!(counts[1] > 0 && counts[2] > 0, "{counts:?}");
    }

    #[test]
    fn brackets_and_quotes_stay_rare() {
        let mut list = vec!["x".to_string(); 2000];
        punctuate(&mut list, &mut rng());

        let wrapped = list
            .iter()
            .filter(|word| word.starts_with('(') || word.starts_with('"'))
            .count();

        // Around one word in forty, not one in ten.
        assert!(wrapped > 0, "nothing was wrapped");
        assert!(
            wrapped < list.len() / 20,
            "{wrapped} of {} wrapped",
            list.len()
        );
    }

    #[test]
    fn a_punctuated_test_opens_on_a_capital() {
        let mut list = words("the cat sat");
        punctuate(&mut list, &mut rng());

        assert!(list[0].starts_with('T'), "{:?}", list[0]);
    }

    #[test]
    fn a_full_stop_capitalises_the_next_word() {
        let mut list = words("one two. three four");
        capitalise_sentences(&mut list);

        assert_eq!(list, words("One two. Three four"));
    }

    #[test]
    fn a_comma_does_not() {
        let mut list = words("one two, three");
        capitalise_sentences(&mut list);

        assert_eq!(list, words("One two, three"));
    }

    #[test]
    fn a_closing_quote_doesnt_hide_the_full_stop() {
        let mut list = words("\"one.\" two");
        capitalise_sentences(&mut list);

        // Capitalised inside the quotes, and the sentence still ended.
        assert_eq!(list, words("\"One.\" Two"));
    }

    #[test]
    fn a_number_starting_a_sentence_is_left_alone() {
        let mut list = words("one. 42 three");
        capitalise_sentences(&mut list);

        assert_eq!(list, words("One. 42 three"));
    }

    #[test]
    fn numbers_never_replace_the_first_word() {
        // A long list and a real generator, so the rule is what keeps the
        // first word rather than the odds.
        for _ in 0..50 {
            let mut list = vec!["the".to_string(); 200];
            numerify(&mut list, &mut rand::rng());

            assert_eq!(list[0], "the");
        }
    }

    #[test]
    fn some_of_the_words_become_numbers() {
        let mut list = vec!["x".to_string(); 200];
        numerify(&mut list, &mut rng());

        let numbers = list.iter().filter(|word| *word != "x").count();
        assert!(numbers > 0, "nothing was numbered");
        // Roughly one in eight, and nowhere near all of them — a test that is
        // mostly numbers is a numeric keypad drill.
        assert!(
            numbers < list.len() / 2,
            "{numbers} of {} numbered",
            list.len()
        );
    }

    #[test]
    fn a_number_is_at_most_four_digits() {
        let mut list = vec!["x".to_string(); 200];
        numerify(&mut list, &mut rng());

        for word in list.iter().filter(|word| *word != "x") {
            assert!(word.len() <= 4, "{word}");
        }
    }
}
