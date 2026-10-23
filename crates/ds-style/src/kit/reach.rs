//! What a stylesheet of the person's or a look's may fetch (customization design section 7.6):
//! `url()`, `@import` and `@font-face` may name `data:` or a file inside the sheet's own folder,
//! and nothing else. Blitz gives a net provider no way to tell which sheet asked, so the rule is
//! a rewrite of the text before it goes into `<style>`: a `url()` that reaches elsewhere becomes
//! an empty `data:` URL (the rule around it stays valid) and an `@import` of such a target is
//! removed. Each cut is recorded, with its line, for the notes.

use std::path::{Component, Path};

/// The URL a cut `url()` is replaced by: it loads nothing.
const EMPTY: &str = "url(\"data:,\")";

/// One reference the rewrite cut.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cut {
    /// The line of the sheet it was on, from 1.
    pub line: usize,
    /// What it named, as written.
    pub target: String,
}

/// What the rewrite does with the text it just measured.
enum Emit {
    /// Keep it.
    Copy,
    /// A `url()` that reaches out: write the empty one instead.
    Replace(String),
    /// An `@import` that reaches out: write nothing.
    Drop(String),
}

/// `css` with every reference outside `home` cut, and the cuts. With no `home`, only `data:` and
/// in-document (`#id`) references stay.
pub fn within(css: &str, home: Option<&Path>) -> (String, Vec<Cut>) {
    let mut out = String::with_capacity(css.len());
    let mut cuts = Vec::new();
    let mut rest = css;
    let mut line = 1;
    let mut prev = ' ';
    while let Some(first) = rest.chars().next() {
        let word_start = !(prev.is_ascii_alphanumeric() || matches!(prev, '-' | '_'));
        let (len, emit) = if rest.starts_with("/*") {
            (comment(rest), Emit::Copy)
        } else if first == '"' || first == '\'' {
            (string(rest), Emit::Copy)
        } else if starts_with_word(rest, "@import") {
            match import(rest, home) {
                (len, Some(target)) => (len, Emit::Drop(target)),
                (len, None) => (len, Emit::Copy),
            }
        } else if word_start && starts_with_word(rest, "url(") {
            match url(rest, home) {
                (len, Some(target)) => (len, Emit::Replace(target)),
                (len, None) => (len, Emit::Copy),
            }
        } else {
            (first.len_utf8(), Emit::Copy)
        };
        let taken = &rest[..len];
        match emit {
            Emit::Copy => out.push_str(taken),
            Emit::Replace(target) => {
                out.push_str(EMPTY);
                cuts.push(Cut { line, target });
            }
            Emit::Drop(target) => cuts.push(Cut { line, target }),
        }
        line += taken.matches('\n').count();
        prev = taken.chars().next_back().unwrap_or(prev);
        rest = &rest[len..];
    }
    (out, cuts)
}

fn starts_with_word(text: &str, word: &str) -> bool {
    text.get(..word.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(word))
}

/// The length of the comment at the start of `text` (to its end, or the end of the text).
fn comment(text: &str) -> usize {
    text[2..].find("*/").map_or(text.len(), |at| at + 4)
}

/// The length of the string at the start of `text` (quote included), honouring `\` escapes.
fn string(text: &str) -> usize {
    let quote = text.as_bytes()[0];
    let bytes = text.as_bytes();
    let mut at = 1;
    while at < bytes.len() {
        match bytes[at] {
            b'\\' => at += 2,
            b if b == quote => return at + 1,
            _ => at += 1,
        }
    }
    text.len()
}

/// The `url(...)` at the start of `text`: its length and, when it reaches outside `home`, what it
/// named. An unterminated one is left alone (the parser drops it).
fn url(text: &str, home: Option<&Path>) -> (usize, Option<String>) {
    let body = &text[4..];
    let trimmed = body.trim_start();
    let (target, after) = match trimmed.chars().next() {
        Some('"' | '\'') => {
            let len = string(trimmed);
            (&trimmed[1..len.saturating_sub(1).max(1)], &trimmed[len..])
        }
        _ => match trimmed.find(')') {
            Some(at) => (trimmed[..at].trim_end(), &trimmed[at..]),
            None => return (text.len(), None),
        },
    };
    let closing = after.trim_start();
    if !closing.starts_with(')') {
        return (text.len(), None);
    }
    let len = text.len() - closing.len() + 1;
    (len, (!allowed(target, home)).then(|| target.to_owned()))
}

/// The `@import` at the start of `text`: its length (to the `;` when it is cut, else just the
/// keyword, so its target is scanned like any other text) and the string target it cuts.
fn import(text: &str, home: Option<&Path>) -> (usize, Option<String>) {
    let keyword = "@import".len();
    let after = &text[keyword..];
    let trimmed = after.trim_start();
    if !trimmed.starts_with(['"', '\'']) {
        return (keyword, None);
    }
    let len = string(trimmed);
    let target = &trimmed[1..len.saturating_sub(1).max(1)];
    if allowed(target, home) {
        return (keyword, None);
    }
    let start = text.len() - trimmed.len();
    let end = trimmed[len..]
        .find(';')
        .map_or(text.len(), |at| start + len + at + 1);
    (end, Some(target.to_owned()))
}

/// Whether `target` stays inside `home`: `data:`, an in-document `#id`, or a path (relative, or
/// absolute / `file:` under `home`) with no way out of it.
fn allowed(target: &str, home: Option<&Path>) -> bool {
    let target = target.trim();
    if target.len() >= 5 && target[..5].eq_ignore_ascii_case("data:") {
        return true;
    }
    if target.starts_with('#') {
        return true;
    }
    // Escapes and encodings could spell a way out that a text check cannot see.
    if target.contains(['%', '\\']) || target.starts_with("//") {
        return false;
    }
    let Some(home) = home else {
        return false;
    };
    let path = match scheme(target) {
        Some(scheme) if scheme.eq_ignore_ascii_case("file") => {
            let rest = &target[scheme.len() + 1..];
            match rest.strip_prefix("//") {
                Some(host_and_path) => host_and_path
                    .strip_prefix("localhost")
                    .unwrap_or(host_and_path),
                None => rest,
            }
        }
        Some(_) => return false,
        None => target,
    };
    let path = path.split(['?', '#']).next().unwrap_or_default();
    let path = Path::new(path);
    let inside = if path.is_absolute() {
        path.starts_with(home)
    } else {
        true
    };
    inside && path.components().all(|part| part != Component::ParentDir)
}

/// The scheme of `target` (`https` of `https://x`), if it starts with one.
fn scheme(target: &str) -> Option<&str> {
    let colon = target.find(':')?;
    let head = &target[..colon];
    let mut chars = head.chars();
    let first = chars.next()?;
    (first.is_ascii_alphabetic()
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.')))
    .then_some(head)
}
