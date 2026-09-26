//! Cutting a paragraph into the words a dictionary is asked about. Pure: the tables below are
//! the whole rule.
//!
//! A word is a run of letters and digits (CJK excluded, see [`super::script`]), with an
//! apostrophe allowed between two letters. It is never checked when it:
//!
//! - is in a URL, an email address or a path: the whitespace-separated chunk around it starts
//!   with a scheme and `://`, `www.`, `mailto:`, `/`, `~/`, `./` or `../`, or is `local@domain.tld`;
//! - is in a code span: between a pair of backticks, or in a stretch the host reports as code
//!   (an element such as `code` or `pre`, [`Paragraph::skips`](super::Paragraph));
//! - is all capitals (`NASA`, `HTTP`), or contains a digit (`mp3`, `2nd`).

use super::script::{is_apostrophe, is_word_char};

/// A stretch of a paragraph's text, as UTF-8 byte offsets (the same offsets as a
/// [`TextPosition`](crate::TextPosition)).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Span {
    /// Where it starts.
    pub start: usize,
    /// Where it ends, exclusive.
    pub end: usize,
}

impl Span {
    /// The stretch from `start` to `end`.
    pub fn new(start: usize, end: usize) -> Span {
        Span { start, end }
    }

    /// Whether the two share a byte.
    pub fn overlaps(self, other: Span) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Whether `offset` is in it or at either end: a caret touching the word.
    pub fn touches(self, offset: usize) -> bool {
        self.start <= offset && offset <= self.end
    }
}

/// The words of `text` a dictionary should check, in order, less any that overlap `skips` (the
/// host's code stretches).
pub fn words(text: &str, skips: &[Span]) -> Vec<Span> {
    let code = code_spans(text);
    chunks(text)
        .filter(|chunk| !is_link(&text[chunk.start..chunk.end]))
        .flat_map(|chunk| runs(text, chunk))
        .filter(|word| !code.iter().chain(skips).any(|skip| skip.overlaps(*word)))
        .filter(|word| checkable(&text[word.start..word.end]))
        .collect()
}

/// The word of `text` the caret at `offset` touches, if any: the one being typed.
pub fn word_at(text: &str, offset: usize) -> Option<Span> {
    chunks(text)
        .flat_map(|chunk| runs(text, chunk))
        .find(|word| word.touches(offset))
}

/// Whether `text` has a word character at the byte before `at` or at `at`: a span ending or
/// starting there is not a whole word.
pub(crate) fn joined_at(text: &str, at: usize) -> bool {
    let before = text.get(..at).and_then(|head| head.chars().next_back());
    let after = text.get(at..).and_then(|tail| tail.chars().next());
    before.is_some_and(is_word_char) && after.is_some_and(is_word_char)
}

/// The whitespace-separated chunks of `text`.
fn chunks(text: &str) -> impl Iterator<Item = Span> + '_ {
    let mut start = None;
    text.char_indices()
        .map(Some)
        .chain(std::iter::once(None))
        .filter_map(move |step| match step {
            Some((at, c)) if c.is_whitespace() => start.take().map(|s| Span::new(s, at)),
            Some((at, _)) => {
                start.get_or_insert(at);
                None
            }
            None => start.take().map(|s| Span::new(s, text.len())),
        })
}

/// The runs of word characters in `chunk`, an apostrophe joining two letters.
fn runs(text: &str, chunk: Span) -> Vec<Span> {
    let chars: Vec<(usize, char)> = text[chunk.start..chunk.end]
        .char_indices()
        .map(|(at, c)| (chunk.start + at, c))
        .collect();
    let mut found = Vec::new();
    let mut start = None;
    for (index, &(at, c)) in chars.iter().enumerate() {
        let joins = is_apostrophe(c)
            && start.is_some()
            && chars
                .get(index + 1)
                .is_some_and(|&(_, next)| next.is_alphabetic());
        if is_word_char(c) || joins {
            start.get_or_insert(at);
        } else if let Some(begun) = start.take() {
            found.push(Span::new(begun, at));
        }
    }
    if let Some(begun) = start {
        found.push(Span::new(begun, chunk.end));
    }
    found
}

/// Whether a word is worth asking a dictionary about: no digit, not all capitals.
fn checkable(word: &str) -> bool {
    let digit = word.chars().any(char::is_numeric);
    let upper = word.chars().any(char::is_uppercase);
    let lower = word.chars().any(char::is_lowercase);
    !digit && !(upper && !lower)
}

/// Whether a chunk is a URL, an email address or a path.
fn is_link(chunk: &str) -> bool {
    let bare = chunk
        .trim_start_matches(['(', '[', '{', '<', '"', '\'', '\u{201C}', '\u{2018}'])
        .trim_end_matches([
            ')', ']', '}', '>', '"', '\'', '.', ',', ';', ':', '!', '?', '\u{201D}', '\u{2019}',
        ]);
    let lower = bare.to_lowercase();
    has_scheme(bare)
        || ["www.", "mailto:", "/", "~/", "./", "../"]
            .iter()
            .any(|prefix| lower.starts_with(prefix))
        || is_email(bare)
}

/// `scheme://…`, the scheme a letter then letters, digits, `+`, `-` or `.`.
fn has_scheme(chunk: &str) -> bool {
    chunk.split_once("://").is_some_and(|(scheme, _)| {
        let mut chars = scheme.chars();
        chars.next().is_some_and(|c| c.is_ascii_alphabetic())
            && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    })
}

/// `local@domain.tld`: one `@`, something before it, and a domain of two or more labels.
fn is_email(chunk: &str) -> bool {
    let mut parts = chunk.split('@');
    let (Some(local), Some(domain), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    let labels: Vec<&str> = domain.split('.').collect();
    !local.is_empty() && labels.len() >= 2 && labels.iter().all(|label| !label.is_empty())
}

/// The stretches between paired backticks, the ticks included; an unpaired tick opens nothing.
fn code_spans(text: &str) -> Vec<Span> {
    let ticks: Vec<usize> = text.match_indices('`').map(|(at, _)| at).collect();
    let (pairs, _) = ticks.as_chunks::<2>();
    pairs
        .iter()
        .map(|[open, close]| Span::new(*open, close + 1))
        .collect()
}

#[cfg(test)]
#[path = "words_tests.rs"]
mod tests;
