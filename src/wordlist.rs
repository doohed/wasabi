use rand::seq::IndexedRandom;

use crate::word::Word;

/// The 200 most common English words, which is what MonkeyType's default test
/// draws from. Short and overwhelmingly ASCII, so lines wrap predictably and
/// the test measures typing rather than reading.
const POOL: [&str; 200] = [
    "the", "be", "of", "and", "a", "to", "in", "he", "have", "it", "that", "for", "they", "I",
    "with", "as", "not", "on", "she", "at", "by", "this", "we", "you", "do", "but", "from", "or",
    "which", "one", "would", "all", "will", "there", "say", "who", "make", "when", "can", "more",
    "if", "no", "man", "out", "other", "so", "what", "time", "up", "go", "about", "than", "into",
    "could", "state", "only", "new", "year", "some", "take", "come", "these", "know", "see", "use",
    "get", "like", "then", "first", "any", "work", "now", "may", "such", "give", "over", "think",
    "most", "even", "find", "day", "also", "after", "way", "many", "must", "look", "before",
    "great", "back", "through", "long", "where", "much", "should", "well", "people", "down", "own",
    "just", "because", "good", "each", "those", "feel", "seem", "how", "high", "too", "place",
    "little", "world", "very", "still", "nation", "hand", "old", "life", "tell", "write", "become",
    "here", "show", "house", "both", "between", "need", "mean", "call", "develop", "under", "last",
    "right", "move", "thing", "general", "school", "never", "same", "another", "begin", "while",
    "number", "part", "turn", "real", "leave", "might", "want", "point", "form", "off", "child",
    "few", "small", "since", "against", "ask", "late", "home", "interest", "large", "person",
    "end", "open", "public", "follow", "during", "present", "without", "again", "hold", "govern",
    "around", "possible", "head", "consider", "word", "program", "problem", "however", "lead",
    "system", "set", "order", "eye", "plan", "run", "keep", "face", "fact", "group", "play",
    "stand", "increase", "early", "course", "change", "help", "line",
];

/// `count` words drawn at random, with repetition.
///
/// With repetition on purpose: sampling without it would bias the tail of a
/// long test towards the rare, awkward words once the common ones are used up.
pub fn random(count: usize) -> Vec<Word> {
    let mut rng = rand::rng();

    (0..count)
        .map(|_| Word::new(POOL.choose(&mut rng).expect("POOL is never empty")))
        .collect()
}
