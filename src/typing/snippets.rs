//! The code the code test deals.
//!
//! Short, complete, and deliberately ordinary: a binary search and a bubble
//! sort are the same shape in every language, so what the test measures is
//! your hands on the punctuation rather than your memory of a clever algorithm.
//!
//! Compiled in rather than read from a file. Unlike the word list, a snippet
//! has to be *valid* — plausible code that doesn't parse would be a worse test
//! than no code test — and checking that is not something this app can do for
//! a file someone hands it.

use rand::seq::IndexedRandom;

use super::mode::Language;
use super::word::Word;

/// Snippets are dealt until the test is this many words long.
///
/// The same rate the word test uses. Code tokens are shorter than words on
/// average, so this overshoots — which is the right way to be wrong: the timer
/// ends the test, never the snippets running out.
use super::wordlist::WORDS_PER_SECOND;

const C: [&str; 3] = [
    "\
int binary_search(const int *xs, int n, int key) {
    int lo = 0;
    int hi = n - 1;
    while (lo <= hi) {
        int mid = lo + (hi - lo) / 2;
        if (xs[mid] == key) {
            return mid;
        }
        if (xs[mid] < key) {
            lo = mid + 1;
        } else {
            hi = mid - 1;
        }
    }
    return -1;
}",
    "\
void bubble_sort(int *xs, int n) {
    for (int i = 0; i < n - 1; i++) {
        for (int j = 0; j < n - i - 1; j++) {
            if (xs[j] > xs[j + 1]) {
                int tmp = xs[j];
                xs[j] = xs[j + 1];
                xs[j + 1] = tmp;
            }
        }
    }
}",
    "\
unsigned long gcd(unsigned long a, unsigned long b) {
    while (b != 0) {
        unsigned long tmp = b;
        b = a % b;
        a = tmp;
    }
    return a;
}",
];

const RUST: [&str; 3] = [
    "\
fn binary_search(xs: &[i32], key: i32) -> Option<usize> {
    let mut lo = 0;
    let mut hi = xs.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        match xs[mid].cmp(&key) {
            Ordering::Equal => return Some(mid),
            Ordering::Less => lo = mid + 1,
            Ordering::Greater => hi = mid,
        }
    }
    None
}",
    "\
fn bubble_sort(xs: &mut [i32]) {
    for i in 0..xs.len() {
        let mut swapped = false;
        for j in 0..xs.len() - i - 1 {
            if xs[j] > xs[j + 1] {
                xs.swap(j, j + 1);
                swapped = true;
            }
        }
        if !swapped {
            break;
        }
    }
}",
    "\
fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let tmp = b;
        b = a % b;
        a = tmp;
    }
    a
}",
];

impl Language {
    /// The snippets this language can deal.
    fn snippets(self) -> &'static [&'static str] {
        match self {
            Language::C => &C,
            Language::Rust => &RUST,
        }
    }
}

/// A test's worth of code for a run of `seconds`.
///
/// Whole snippets, one after another, until there is enough of it; the last
/// one is cut off wherever the count lands, because the clock was always going
/// to stop you somewhere. Drawn at random each time, so a second run at the
/// same length isn't the same test.
pub fn deal(language: Language, seconds: u64) -> Vec<Word> {
    let mut rng = rand::rng();
    let snippets = language.snippets();
    let count = seconds as usize * WORDS_PER_SECOND;

    let mut words = Vec::new();
    while words.len() < count {
        let snippet = snippets.choose(&mut rng).expect("every language has some");
        words.extend(tokenise(snippet));
    }

    words.truncate(count);
    words
}

/// Split a snippet into the words a test is made of.
///
/// One word per run of non-whitespace, and the first word of each line carries
/// that line's indentation. Indentation is *drawn*, never typed: lining code up
/// is the editor's job in real life, and a test that charged you for spaces
/// would be measuring your patience.
///
/// Blank lines are dropped rather than kept as empty rows. They separate ideas
/// in a file, and there is nothing to type on one.
fn tokenise(code: &str) -> Vec<Word> {
    let mut words = Vec::new();

    for line in code.lines() {
        let body = line.trim_start();
        if body.is_empty() {
            continue;
        }

        let indent = line.chars().count() - body.chars().count();

        for (position, token) in body.split_whitespace().enumerate() {
            words.push(match position {
                0 => Word::at_indent(token, indent as u16),
                _ => Word::new(token),
            });
        }
    }

    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_becomes_its_tokens() {
        let words = tokenise("let x = 1;");
        let targets: Vec<&str> = words.iter().map(|word| word.target.as_str()).collect();

        assert_eq!(targets, ["let", "x", "=", "1;"]);
    }

    #[test]
    fn only_the_first_word_of_a_line_carries_the_indent() {
        let words = tokenise("fn f() {\n    let x = 1;\n}");

        assert_eq!(words[0].indent, Some(0)); // fn
        assert_eq!(words[1].indent, None); // f()
        assert_eq!(words[3].target, "let");
        assert_eq!(words[3].indent, Some(4));
        assert_eq!(words[4].indent, None); // x
        assert_eq!(words.last().unwrap().target, "}");
        assert_eq!(words.last().unwrap().indent, Some(0));
    }

    #[test]
    fn a_blank_line_is_not_a_word() {
        let words = tokenise("a\n\n   \nb");
        let targets: Vec<&str> = words.iter().map(|word| word.target.as_str()).collect();

        assert_eq!(targets, ["a", "b"]);
        assert_eq!(words[1].indent, Some(0));
    }

    #[test]
    fn a_deal_is_long_enough_for_the_clock() {
        for language in Language::ALL {
            let words = deal(language, 15);
            assert_eq!(words.len(), 15 * WORDS_PER_SECOND);
        }
    }

    #[test]
    fn a_deal_starts_at_the_beginning_of_a_snippet() {
        // Cut off at the end, never the start: a test opening mid-expression
        // would read as a bug.
        let words = deal(Language::Rust, 15);
        assert_eq!(words[0].indent, Some(0));
    }

    #[test]
    fn every_snippet_has_something_to_type() {
        for language in Language::ALL {
            for snippet in language.snippets() {
                let words = tokenise(snippet);

                assert!(words.len() > 5, "{} snippet is tiny", language.name());
                // Every line starts somewhere, and the first one at column 0.
                assert_eq!(words[0].indent, Some(0));
            }
        }
    }

    #[test]
    fn no_snippet_line_is_too_wide_for_the_test_column() {
        // The typing column is 72 columns; a line longer than that wraps, and
        // wrapped code stops looking like code.
        for language in Language::ALL {
            for snippet in language.snippets() {
                for line in snippet.lines() {
                    let width = line.chars().count();
                    assert!(
                        width <= 60,
                        "{width} columns in {}: {line}",
                        language.name()
                    );
                }
            }
        }
    }
}
