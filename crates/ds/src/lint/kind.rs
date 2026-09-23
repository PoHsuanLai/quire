//! Classifying a [`super::tokenize::Located`] token by the shape of its own text.
//!
//! `Located` carries only the token's source text, not `cssparser`'s `Token` enum (that type
//! borrows from the input, and threading its lifetime through the rule engine would mean every
//! rule function carries it too). CSS's lexical grammar makes every token's *kind* recoverable
//! from its own spelling alone — a hash token always starts with `#`, a function token always
//! ends in `(` with a name before it, a dimension is a number immediately followed by a unit —
//! so re-deriving the kind here is not the "substring on raw source" pattern CONVENTIONS warns
//! against: it classifies one already-tokenized atom, never searches for a fragment inside a
//! larger string.

/// Whether a token is whitespace or a comment: never itself part of a rule.
pub fn is_trivial(text: &str) -> bool {
    text.starts_with("/*") || text.chars().all(char::is_whitespace)
}

/// The first non-trivial token at or after `tokens[from]`.
///
/// Used wherever CSS grammar allows optional whitespace between two tokens that a rule cares
/// about (`! important`, `var( --x )`, the first token after a `;`) — the handful of places
/// [`is_trivial`] alone is not enough because a plain `tokens[i + 1]` would land on the
/// whitespace instead of the token past it.
pub fn next_significant(
    tokens: &[super::tokenize::Located],
    from: usize,
) -> Option<&super::tokenize::Located> {
    tokens[from.min(tokens.len())..]
        .iter()
        .find(|t| !is_trivial(&t.text))
}

/// `tokens` with any leading or trailing whitespace/comment tokens dropped; whitespace in the
/// middle (a descendant combinator, the space in `1px solid`) is left alone, since it is part
/// of what the excerpt or selector text means.
pub fn trim_trivia(tokens: &[super::tokenize::Located]) -> &[super::tokenize::Located] {
    let start = tokens
        .iter()
        .position(|t| !is_trivial(&t.text))
        .unwrap_or(tokens.len());
    let end = tokens
        .iter()
        .rposition(|t| !is_trivial(&t.text))
        .map_or(start, |p| p + 1);
    &tokens[start..end]
}

/// The name of a `Function` token (`"rgba("` -> `Some("rgba")`), or `None` for a bare `(`.
pub fn function_name(text: &str) -> Option<&str> {
    let name = text.strip_suffix('(')?;
    (!name.is_empty()).then_some(name)
}

/// A `Hash`/`IDHash` token's text (`#fff`, `#112233`), unchanged (it already carries the `#`).
pub fn is_hash(text: &str) -> bool {
    let rest = match text.strip_prefix('#') {
        Some(rest) => rest,
        None => return false,
    };
    matches!(rest.len(), 3 | 4 | 6 | 8) && rest.chars().all(|c| c.is_ascii_hexdigit())
}

/// Whether `text` is a plain identifier: not a hash, not a function, not punctuation, not a
/// quoted string, not a number/dimension/percentage.
pub fn is_ident(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_alphabetic() || first == '_' || first == '-') {
        return false;
    }
    if first == '-' {
        // A single `-` is the `Delim('-')` token, not an ident; `--x` and `-x` are idents.
        if text.len() == 1 {
            return false;
        }
        let second = chars.clone().next();
        if second.is_some_and(|c| c.is_ascii_digit()) {
            return false;
        }
    }
    text.chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || !c.is_ascii())
}

/// A dimension token (`"200ms"`, `"-4px"`) split into its numeric prefix and unit suffix.
pub fn dimension_unit(text: &str) -> Option<&str> {
    let bytes = text.as_bytes();
    let mut index = 0;
    if matches!(bytes.first(), Some(b'+' | b'-')) {
        index += 1;
    }
    let digits_start = index;
    while bytes.get(index).is_some_and(u8::is_ascii_digit) {
        index += 1;
    }
    if bytes.get(index) == Some(&b'.') {
        let after_dot = index + 1;
        if bytes.get(after_dot).is_some_and(u8::is_ascii_digit) {
            index = after_dot;
            while bytes.get(index).is_some_and(u8::is_ascii_digit) {
                index += 1;
            }
        }
    }
    if index == digits_start {
        // No digits at all: not a number, so not a dimension.
        return None;
    }
    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        let mut exp = index + 1;
        if matches!(bytes.get(exp), Some(b'+' | b'-')) {
            exp += 1;
        }
        if bytes.get(exp).is_some_and(u8::is_ascii_digit) {
            index = exp;
            while bytes.get(index).is_some_and(u8::is_ascii_digit) {
                index += 1;
            }
        }
    }
    let unit = &text[index..];
    (!unit.is_empty()).then_some(unit)
}

/// A bare number token (`"14"`, `"-1.5"`): a numeric prefix with nothing after it.
pub fn is_number(text: &str) -> bool {
    !text.ends_with('%') && dimension_unit(text).is_none() && has_numeric_prefix(text)
}

/// A percentage token (`"50%"`).
pub fn is_percentage(text: &str) -> bool {
    text.strip_suffix('%')
        .is_some_and(|rest| has_numeric_prefix(rest) && dimension_unit(rest).is_none())
}

fn has_numeric_prefix(text: &str) -> bool {
    let mut chars = text.chars().peekable();
    if let Some('+' | '-') = chars.peek() {
        chars.next();
    }
    let mut saw_digit = false;
    while chars.peek().is_some_and(char::is_ascii_digit) {
        chars.next();
        saw_digit = true;
    }
    if chars.peek() == Some(&'.') {
        chars.next();
        while chars.peek().is_some_and(char::is_ascii_digit) {
            chars.next();
            saw_digit = true;
        }
    }
    saw_digit && chars.next().is_none()
}
