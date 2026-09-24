//! What a `TextInput` holds: a line of text, a secret, a file's name, or several lines
//! (mailo gaps 4 added `Secret`, `File` and `Multiline`).

/// How many lines a multiline field shows: its `rows`, and its least height when it grows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rows(pub u8);

/// Whether a multiline field grows with what is typed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Grow {
    /// Always `rows` lines; more scroll inside it.
    #[default]
    Fixed,
    /// At least `rows` lines, and one more for each line the value has beyond them. Only hard
    /// line breaks count: the field cannot measure a soft wrap before it is laid out, so a long
    /// line that wraps scrolls inside the field instead of growing it.
    ToContent,
}

/// What the field holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextInputKind {
    /// `type="text"`.
    #[default]
    Text,
    /// `type="password"`, controlled like any field: its `value` is written into the markup.
    /// Blitz lays a password field out as text and draws its characters as typed, so the field
    /// paints its text transparent and lays a row of dots over it, one per character.
    Password,
    /// A secret that never reaches the markup: the field keeps what is typed in its own state
    /// and hands it out only through `oninput` and `onchange`. No `value` attribute is ever
    /// written (the `value` prop is ignored), only one dot per character over the caret. To
    /// clear it, remount the field under a new `key`.
    Secret,
    /// A file's name and a "Choose…" button. There is no native picker on Blitz, so the field
    /// only asks: `on_pick` fires, the host opens its own chooser (a portal, a sheet), and
    /// passes back the chosen name as `value`.
    File,
    /// A `textarea` of `rows` lines that may grow with its content.
    Multiline {
        /// The lines shown, and the least when it grows.
        rows: Rows,
        /// Whether it grows with hard line breaks.
        grow: Grow,
    },
}

/// One masked character.
const MASK_DOT: char = '\u{2022}';

impl TextInputKind {
    /// The `type` attribute of the single-line `input`.
    pub(crate) fn input_type(self) -> &'static str {
        match self {
            TextInputKind::Password | TextInputKind::Secret => "password",
            TextInputKind::Text | TextInputKind::File | TextInputKind::Multiline { .. } => "text",
        }
    }

    /// `data-kind`: written for every kind but text, so a text field's markup is as it was.
    pub(crate) fn data_kind(self) -> Option<&'static str> {
        match self {
            TextInputKind::Text => None,
            TextInputKind::Password => Some("password"),
            TextInputKind::Secret => Some("secret"),
            TextInputKind::File => Some("file"),
            TextInputKind::Multiline { .. } => Some("multiline"),
        }
    }

    /// The dots drawn over a masked value; nothing for an unmasked kind or an empty value.
    pub(crate) fn mask(self, value: &str) -> Option<String> {
        match self {
            TextInputKind::Password | TextInputKind::Secret if !value.is_empty() => {
                Some(value.chars().map(|_| MASK_DOT).collect())
            }
            _ => None,
        }
    }
}

impl Grow {
    /// The `rows` a multiline field with `value` is drawn at: `rows`, or under `ToContent` the
    /// value's line count when that is more (saturating at 255).
    pub fn rows(self, rows: Rows, value: &str) -> Rows {
        match self {
            Grow::Fixed => rows,
            Grow::ToContent => {
                let lines = value.split('\n').count();
                Rows(rows.0.max(u8::try_from(lines).unwrap_or(u8::MAX)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Grow, Rows, TextInputKind};

    #[test]
    fn a_growing_field_takes_one_row_per_line_beyond_its_least() {
        const CASES: &[(Grow, u8, &str, u8)] = &[
            (Grow::Fixed, 3, "a\nb\nc\nd", 3),
            (Grow::ToContent, 3, "", 3),
            (Grow::ToContent, 3, "a\nb", 3),
            (Grow::ToContent, 3, "a\nb\nc\nd", 4),
            (Grow::ToContent, 1, "a\n", 2),
        ];
        for &(grow, least, value, want) in CASES {
            assert_eq!(
                grow.rows(Rows(least), value),
                Rows(want),
                "{grow:?} {value:?}"
            );
        }
    }

    #[test]
    fn only_the_secret_kinds_are_masked() {
        const CASES: &[(TextInputKind, &str, Option<&str>)] = &[
            (TextInputKind::Text, "abc", None),
            (
                TextInputKind::Password,
                "abc",
                Some("\u{2022}\u{2022}\u{2022}"),
            ),
            (TextInputKind::Secret, "ab", Some("\u{2022}\u{2022}")),
            (TextInputKind::Secret, "", None),
            (TextInputKind::File, "a.pdf", None),
        ];
        for &(kind, value, want) in CASES {
            assert_eq!(kind.mask(value).as_deref(), want, "{kind:?}");
        }
    }
}
