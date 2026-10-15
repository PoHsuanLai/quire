//! The rule that tells a switch from a choice between two things (detent's `TogglePair`, design/22
//! section 9.1).

/// Words that are the off side of a pair.
const OFF_WORDS: [&str; 5] = ["off", "hide", "never", "none", "nothing"];

/// Pairs that are a switch though neither word is off: (on, off).
const ON_OFF_PAIRS: [(&str, &str); 3] = [
    ("natural", "traditional"),
    ("reduced", "standard"),
    ("reduced", "system"),
];

/// A two-word key that is a switch: which word is on and which off.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Pair {
    pub on: String,
    pub off: String,
}

impl Pair {
    /// The switch `words` make, or `None` when they are a choice between two things.
    pub(crate) fn of(words: &[String; 2]) -> Option<Pair> {
        let [first, second] = words;
        let pair = |on: &String, off: &String| Pair {
            on: on.clone(),
            off: off.clone(),
        };
        let off = |word: &String| OFF_WORDS.contains(&word.as_str());
        match (off(first), off(second)) {
            (false, true) => return Some(pair(first, second)),
            (true, false) => return Some(pair(second, first)),
            (true, true) => return None,
            (false, false) => {}
        }
        ON_OFF_PAIRS
            .iter()
            .find_map(|(on, off)| match (first.as_str(), second.as_str()) {
                (a, b) if (a, b) == (*on, *off) => Some(pair(first, second)),
                (a, b) if (a, b) == (*off, *on) => Some(pair(second, first)),
                _ => None,
            })
    }
}
