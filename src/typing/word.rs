/// How a single character should be displayed.
///
/// Every character shown in the typing area is in exactly one of these states.
/// This enum is the entire contract between the domain model and the renderer:
/// `word.rs` decides the states, `ui/typing.rs` decides the colours, and
/// neither needs to know anything else about the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharState {
    /// Typed, and matches the expected character.
    Correct,
    /// Typed, but does not match the expected character.
    Incorrect,
    /// Typed past the end of the word — there is no expected character at all.
    Extra,
    /// Part of the target word the user has not reached yet.
    Untyped,
}

/// One word of the test: what the user is supposed to type, and what they
/// actually typed for it.
///
/// Keeping `typed` per-word (rather than one flat buffer for the whole test)
/// is what makes overflow and word boundaries behave: extra
/// characters pile up at the end of *this* word instead of shifting every
/// character after it.
#[derive(Debug, Clone)]
pub struct Word {
    pub target: String,
    pub typed: String,
    /// `Some(n)` when this word begins a line, indented `n` columns.
    ///
    /// Only the code test sets it — a word test is one long stream, and where
    /// it breaks is the terminal's business. The indent is drawn, never typed:
    /// see [`crate::typing::snippets::tokenise`].
    pub indent: Option<u16>,
}

impl Word {
    /// An untouched word.
    pub fn new(target: &str) -> Self {
        Self {
            target: target.to_string(),
            typed: String::new(),
            indent: None,
        }
    }

    /// An untouched word that begins a line, indented `indent` columns.
    pub fn at_indent(target: &str, indent: u16) -> Self {
        Self {
            indent: Some(indent),
            ..Self::new(target)
        }
    }

    /// A word with a pre-filled `typed` value, so a test can set up a state
    /// without replaying the keystrokes that would reach it.
    #[cfg(test)]
    pub fn with_typed(target: &str, typed: &str) -> Self {
        Self {
            typed: typed.to_string(),
            ..Self::new(target)
        }
    }

    /// Characters this word contributes to the score.
    ///
    /// All of them or none: a word with a mistake anywhere in it scores
    /// nothing, however much of it was right.
    ///
    /// `partial` credits what has been typed so far of an *unfinished* word,
    /// as long as it is still a correct prefix. Only the word under the caret
    /// gets that: the clock cutting you off mid-word isn't your mistake.
    pub fn scoring_chars(&self, partial: bool) -> usize {
        if self.typed == self.target {
            self.target.chars().count()
        } else if partial && self.target.starts_with(&self.typed) {
            self.typed.chars().count()
        } else {
            0
        }
    }

    /// Characters standing in the text, right or wrong — everything except
    /// the parts of the target never reached.
    pub fn standing_chars(&self) -> usize {
        self.char_states()
            .iter()
            .filter(|(_, state)| *state != CharState::Untyped)
            .count()
    }

    /// Classify every character that should be drawn for this word.
    ///
    /// The returned `char` is the one to *display*, which is not always the one
    /// the user pressed:
    ///
    /// - `Correct`, `Incorrect`, `Untyped` yield the **expected** character.
    ///   Showing the expected character on a mistake keeps the line width
    ///   stable, so the text never shifts under the user's fingers.
    /// - `Extra` yields the **typed** character, because there is no expected
    ///   one to show.
    pub fn char_states(&self) -> Vec<(char, CharState)> {
        let mut out = Vec::new();
        let mut target = self.target.chars();
        let mut typed = self.typed.chars();

        loop {
            match (target.next(), typed.next()) {
                // Both present: straight comparison.
                (Some(expected), Some(actual)) => {
                    let state = if expected == actual {
                        CharState::Correct
                    } else {
                        CharState::Incorrect
                    };
                    out.push((expected, state));
                }
                // Target left over: the user hasn't got this far.
                (Some(expected), None) => out.push((expected, CharState::Untyped)),
                // Typed left over: overflow past the end of the word.
                (None, Some(actual)) => out.push((actual, CharState::Extra)),
                // Both exhausted.
                (None, None) => break,
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn states(target: &str, typed: &str) -> Vec<CharState> {
        Word::with_typed(target, typed)
            .char_states()
            .into_iter()
            .map(|(_, state)| state)
            .collect()
    }

    #[test]
    fn untouched_word_is_all_untyped() {
        assert_eq!(states("cat", ""), vec![CharState::Untyped; 3]);
    }

    #[test]
    fn perfect_word_is_all_correct() {
        assert_eq!(states("cat", "cat"), vec![CharState::Correct; 3]);
    }

    #[test]
    fn partial_word_leaves_a_tail() {
        assert_eq!(
            states("cat", "ca"),
            vec![CharState::Correct, CharState::Correct, CharState::Untyped]
        );
    }

    #[test]
    fn wrong_letter_in_the_middle() {
        assert_eq!(
            states("cat", "cxt"),
            vec![CharState::Correct, CharState::Incorrect, CharState::Correct]
        );
    }

    #[test]
    fn overflow_becomes_extra() {
        assert_eq!(
            states("cat", "catsss"),
            vec![
                CharState::Correct,
                CharState::Correct,
                CharState::Correct,
                CharState::Extra,
                CharState::Extra,
                CharState::Extra,
            ]
        );
    }

    #[test]
    fn incorrect_shows_the_expected_character() {
        let word = Word::with_typed("cat", "cxt");
        let chars: String = word.char_states().into_iter().map(|(c, _)| c).collect();
        assert_eq!(chars, "cat");
    }
}
