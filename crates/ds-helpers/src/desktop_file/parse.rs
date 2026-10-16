//! The lines of a desktop file, and the groups they build.

use super::file::DesktopFile;
use super::group::DesktopGroup;
use std::collections::BTreeMap;

/// What one line of the file is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Line<'a> {
    /// Blank, or a `#` comment.
    Blank,
    /// `[Name]`.
    Header(&'a str),
    /// `Key=value` or `Key[locale]=value`, the value as written.
    Pair {
        key: &'a str,
        locale: Option<&'a str>,
        value: &'a str,
    },
    /// Anything else.
    Skipped,
}

/// A key as the line spells it: a name, or a name with `[locale]`. None for an empty name or
/// locale and for brackets that do not pair.
fn split_key(key: &str) -> Option<(&str, Option<&str>)> {
    let brackets = |part: &str| part.contains(['[', ']']);
    match key.split_once('[') {
        None => (!key.is_empty() && !brackets(key)).then_some((key, None)),
        Some((name, rest)) => {
            let locale = rest.strip_suffix(']')?;
            let sound = !name.is_empty() && !locale.is_empty() && !brackets(locale);
            sound.then_some((name, Some(locale)))
        }
    }
}

/// What `line` is. Space around the line, the key and the `=` is ignored, as the specification
/// asks.
fn classify(line: &str) -> Line<'_> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Line::Blank;
    }
    if let Some(inner) = line.strip_prefix('[') {
        return match inner.strip_suffix(']') {
            Some(name) if !name.is_empty() && !name.contains(['[', ']']) => Line::Header(name),
            _ => Line::Skipped,
        };
    }
    let Some((key, value)) = line.split_once('=') else {
        return Line::Skipped;
    };
    match split_key(key.trim()) {
        Some((key, locale)) => Line::Pair {
            key,
            locale,
            value: value.trim(),
        },
        None => Line::Skipped,
    }
}

/// The file `text` holds; a byte order mark at the start is ignored.
pub(super) fn parse_text(text: &str) -> DesktopFile {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut file = DesktopFile::default();
    let mut index: BTreeMap<String, usize> = BTreeMap::new();
    let mut current: Option<usize> = None;
    for line in text.lines() {
        match classify(line) {
            Line::Blank => {}
            Line::Header(name) => {
                let at = *index.entry(name.to_owned()).or_insert_with(|| {
                    file.groups.push(DesktopGroup::named(name));
                    file.groups.len() - 1
                });
                current = Some(at);
            }
            Line::Pair { key, locale, value } => {
                match current.and_then(|at| file.groups.get_mut(at)) {
                    Some(group) => group.set(key, locale, value),
                    None => file.skipped += 1,
                }
            }
            Line::Skipped => file.skipped += 1,
        }
    }
    file
}

#[cfg(test)]
mod tests {
    use super::{Line, classify};

    #[test]
    fn a_line_is_a_comment_a_header_a_pair_or_skipped() {
        // `(case, line, what it is)`.
        const CASES: &[(&str, &str, Line<'static>)] = &[
            ("empty", "", Line::Blank),
            ("spaces only", "   \t", Line::Blank),
            ("comment", "# Name=x", Line::Blank),
            ("indented comment", "  # Name=x", Line::Blank),
            ("header", "[Desktop Entry]", Line::Header("Desktop Entry")),
            (
                "header with space around",
                "  [Desktop Action new]  ",
                Line::Header("Desktop Action new"),
            ),
            (
                "header without the closing bracket",
                "[Desktop Entry",
                Line::Skipped,
            ),
            ("empty header", "[]", Line::Skipped),
            ("header with trailing text", "[A] b", Line::Skipped),
            ("nested brackets in a header", "[A[b]]", Line::Skipped),
            (
                "pair",
                "Name=Files",
                Line::Pair {
                    key: "Name",
                    locale: None,
                    value: "Files",
                },
            ),
            (
                "space around the equals sign",
                "Name = Files ",
                Line::Pair {
                    key: "Name",
                    locale: None,
                    value: "Files",
                },
            ),
            (
                "the value keeps later equals signs",
                "Exec=env A=b run",
                Line::Pair {
                    key: "Exec",
                    locale: None,
                    value: "env A=b run",
                },
            ),
            (
                "empty value",
                "Comment=",
                Line::Pair {
                    key: "Comment",
                    locale: None,
                    value: "",
                },
            ),
            (
                "localized key",
                "Name[de]=Dateien",
                Line::Pair {
                    key: "Name",
                    locale: Some("de"),
                    value: "Dateien",
                },
            ),
            (
                "localized key with country and modifier",
                "Name[sr_RS@latin]=Fajlovi",
                Line::Pair {
                    key: "Name",
                    locale: Some("sr_RS@latin"),
                    value: "Fajlovi",
                },
            ),
            ("empty locale", "Name[]=x", Line::Skipped),
            ("unclosed locale", "Name[de=x", Line::Skipped),
            ("stray closing bracket", "Name]=x", Line::Skipped),
            ("nested locale brackets", "Name[d[e]]=x", Line::Skipped),
            ("empty key", "=x", Line::Skipped),
            ("empty key with a locale only", "[de]=x", Line::Skipped),
            ("no equals sign", "Name", Line::Skipped),
        ];
        for (case, line, want) in CASES {
            assert_eq!(classify(line), *want, "{case}");
        }
    }
}
