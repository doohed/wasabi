//! Which keys went wrong, and the worst of them.
//!
//! The totals say what the mistakes cost; they don't say what to practise.
//! This files every keystroke against the character it was supposed to be, so
//! the results can name the handful of keys that did the damage.

use std::collections::BTreeMap;

/// How many keys the results name.
///
/// Three fits a row beside its label, and a list long enough to need reading
/// is a list nobody acts on.
pub const WORST: usize = 3;

/// One character's record over a run.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Count {
    /// Keystrokes where this was the character the user was meant to type.
    attempts: usize,
    /// How many of those they got wrong.
    misses: usize,
}

/// A character the run got wrong, and how often.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Miss {
    /// The character that was *expected* — the key worth practising.
    pub key: char,
    pub misses: usize,
    /// Times the run asked for this key at all, right or wrong. Not shown, but
    /// it is what separates two keys missed the same number of times.
    pub attempts: usize,
}

/// Every character a run asked for, and how it went.
#[derive(Debug, Clone, Default)]
pub struct Misses {
    counts: BTreeMap<char, Count>,
}

impl Misses {
    /// Throw the run away. Called on restart, like every other run statistic.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// File one keystroke against the character it was supposed to be.
    ///
    /// Keyed on the *expected* character rather than the one pressed: hitting
    /// `b` where the word wanted `v` is a `v` you can practise, where the `b`
    /// is only where the finger landed that time. A keystroke with no expected
    /// character — overflow past the end of a word — belongs to no key at all,
    /// so the caller doesn't file it here.
    pub fn record(&mut self, expected: char, hit: bool) {
        let count = self.counts.entry(expected).or_default();

        count.attempts += 1;
        if !hit {
            count.misses += 1;
        }
    }

    /// The `n` characters that went wrong most, worst first.
    ///
    /// Ranked by what each one cost, not by how often it went wrong per
    /// attempt: over a run this short a rate puts a letter fumbled once out of
    /// one above one fumbled six times out of sixty, which is the wrong thing
    /// to send someone away to practise. A tie goes to the rarer character —
    /// the same judgement, applied to the one figure left to separate them.
    pub fn worst(&self, n: usize) -> Vec<Miss> {
        let mut worst: Vec<Miss> = self
            .counts
            .iter()
            .filter(|(_, count)| count.misses > 0)
            .map(|(key, count)| Miss {
                key: *key,
                misses: count.misses,
                attempts: count.attempts,
            })
            .collect();

        // `key` last, so the order is total: two keys alike in both figures
        // still can't swap places between one frame and the next.
        worst.sort_by(|a, b| {
            b.misses
                .cmp(&a.misses)
                .then(a.attempts.cmp(&b.attempts))
                .then(a.key.cmp(&b.key))
        });
        worst.truncate(n);

        worst
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `misses` files `expected`, hit or missed, `count` times over.
    fn record(misses: &mut Misses, expected: char, hit: bool, count: usize) {
        for _ in 0..count {
            misses.record(expected, hit);
        }
    }

    fn keys(misses: &Misses, n: usize) -> Vec<char> {
        misses.worst(n).into_iter().map(|miss| miss.key).collect()
    }

    #[test]
    fn a_run_with_nothing_recorded_names_no_keys() {
        assert!(Misses::default().worst(WORST).is_empty());
    }

    #[test]
    fn a_key_that_was_always_hit_is_not_a_problem() {
        let mut misses = Misses::default();
        record(&mut misses, 'e', true, 20);

        assert!(misses.worst(WORST).is_empty());
    }

    #[test]
    fn keys_are_ranked_by_what_they_cost() {
        let mut misses = Misses::default();
        record(&mut misses, 'a', false, 1);
        record(&mut misses, 'b', false, 3);
        record(&mut misses, 'c', false, 2);

        assert_eq!(keys(&misses, WORST), vec!['b', 'c', 'a']);
    }

    #[test]
    fn a_tie_goes_to_the_key_typed_less_often() {
        let mut misses = Misses::default();
        // Both missed twice, but `v` came up twice as often to be missed.
        record(&mut misses, 'e', true, 30);
        record(&mut misses, 'e', false, 2);
        record(&mut misses, 'v', true, 1);
        record(&mut misses, 'v', false, 2);

        assert_eq!(keys(&misses, WORST), vec!['v', 'e']);
    }

    #[test]
    fn only_the_worst_n_are_named() {
        let mut misses = Misses::default();
        for key in "abcde".chars() {
            record(&mut misses, key, false, 1);
        }

        assert_eq!(misses.worst(2).len(), 2);
    }

    #[test]
    fn a_miss_carries_both_figures() {
        let mut misses = Misses::default();
        record(&mut misses, 'v', true, 4);
        record(&mut misses, 'v', false, 1);

        assert_eq!(
            misses.worst(WORST),
            vec![Miss {
                key: 'v',
                misses: 1,
                attempts: 5,
            }]
        );
    }

    #[test]
    fn clearing_forgets_the_run() {
        let mut misses = Misses::default();
        record(&mut misses, 'v', false, 3);
        misses.clear();

        assert!(misses.worst(WORST).is_empty());
    }
}
