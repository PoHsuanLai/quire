//! The value escapes of the specification: `\s` a space, `\n`, `\t`, `\r`, `\\`, and in a list
//! `\;` a semicolon that is part of an item.

use std::borrow::Cow;

/// Where a backslash sits: in a plain string, or in an item of a `;` list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Context {
    Text,
    ListItem,
}

/// What `\` followed by `next` stands for, or none when it is no escape (the backslash is then
/// kept as written).
fn escaped(next: char, context: Context) -> Option<char> {
    match (next, context) {
        ('s', _) => Some(' '),
        ('n', _) => Some('\n'),
        ('t', _) => Some('\t'),
        ('r', _) => Some('\r'),
        ('\\', _) => Some('\\'),
        (';', Context::ListItem) => Some(';'),
        _ => None,
    }
}

/// `raw` with its escapes taken; borrowed when it has none.
pub(super) fn unescape(raw: &str) -> Cow<'_, str> {
    if !raw.contains('\\') {
        return Cow::Borrowed(raw);
    }
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        let next = chars.peek().copied();
        match (c, next) {
            ('\\', Some(next)) if escaped(next, Context::Text).is_some() => {
                out.extend(escaped(next, Context::Text));
                chars.next();
            }
            _ => out.push(c),
        }
    }
    Cow::Owned(out)
}

/// The items of a `;` list, escapes taken. The list's closing `;` ends the last item, and an empty
/// item is dropped.
pub(super) fn split_list(raw: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut item = String::new();
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        let next = chars.peek().copied();
        match (c, next) {
            (';', _) => {
                if !item.is_empty() {
                    items.push(std::mem::take(&mut item));
                }
            }
            ('\\', Some(next)) if escaped(next, Context::ListItem).is_some() => {
                item.extend(escaped(next, Context::ListItem));
                chars.next();
            }
            _ => item.push(c),
        }
    }
    if !item.is_empty() {
        items.push(item);
    }
    items
}

#[cfg(test)]
mod tests {
    use super::{split_list, unescape};

    #[test]
    fn a_string_takes_its_escapes() {
        // `(case, raw, text)`.
        const CASES: &[(&str, &str, &str)] = &[
            ("plain text is left alone", "Text Editor", "Text Editor"),
            ("empty", "", ""),
            ("\\s is a space", "a\\sb", "a b"),
            ("\\s keeps a leading and a trailing space", "\\sa\\s", " a "),
            ("\\n, \\t and \\r", "a\\nb\\tc\\rd", "a\nb\tc\rd"),
            ("\\\\ is one backslash", "C:\\\\dir", "C:\\dir"),
            ("\\\\ then s is not \\s", "\\\\s", "\\s"),
            ("an unknown escape is kept as written", "a\\qb", "a\\qb"),
            (r"\; is no escape in a plain string", r"a\;b", r"a\;b"),
            ("a lone backslash at the end is kept", "abc\\", "abc\\"),
            ("a backslash before a multi-byte char is kept", "\\é", "\\é"),
        ];
        for (case, raw, want) in CASES {
            assert_eq!(unescape(raw), *want, "{case}");
        }
    }

    #[test]
    fn a_list_splits_on_semicolons_that_are_not_escaped() {
        // `(case, raw, items)`.
        const CASES: &[(&str, &str, &[&str])] = &[
            ("empty", "", &[]),
            ("one item with no closing semicolon", "a", &["a"]),
            (
                "the closing semicolon ends the last item",
                "a;b;",
                &["a", "b"],
            ),
            ("no closing semicolon", "a;b", &["a", "b"]),
            ("an empty item is dropped", "a;;b;", &["a", "b"]),
            ("only semicolons", ";;", &[]),
            (r"\; stays inside its item", r"a\;b;c;", &["a;b", "c"]),
            (r"\s and \\ inside items", r"a\sb;c\\;", &["a b", "c\\"]),
            (
                "an escaped backslash does not escape the semicolon",
                r"a\\;b",
                &["a\\", "b"],
            ),
            ("an unknown escape is kept", "a\\q;b", &["a\\q", "b"]),
            ("spaces around items are kept", " a ; b ", &[" a ", " b "]),
        ];
        for (case, raw, want) in CASES {
            assert_eq!(split_list(raw), *want, "{case}");
        }
    }
}
