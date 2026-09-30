//! What a `TextField` holds and how it is drawn: its kind, its bezel and whether what it holds
//! is acceptable (design/30 section 2.2). Data, and the pure decisions that follow from it.

use crate::components::content::text_runs::TextLine;
use ds_core::word::Word;
use ds_motion::detail::{detailed::Detailed, moment::Moment, stamp::EventStamp};

/// What the field holds (`NSTextField`, `NSSecureTextField`, `NSSearchField`), `data-kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum FieldKind {
    /// A line of text.
    #[default]
    Plain,
    /// A secret that never reaches the markup: the field keeps what is typed in its own state
    /// and hands it out only through `oninput` and `onchange`. No `value` attribute is ever
    /// written (the `value` prop is ignored), only one dot per character over the caret. To
    /// clear it, remount the field under a new `key`. Blitz lays a password field out as text and
    /// draws its characters as typed, so the field paints its text transparent and lays a row of
    /// dots over it.
    Secure,
    /// A search field: a magnifier before the text and a clear button after it while there is
    /// text, with the operators that narrow the search as tokens under it.
    Search,
}

/// How the field's edge is drawn (`NSTextField.isBezeled`), `data-variant`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum FieldBezel {
    /// A rounded rectangle with a hairline and the surface ground, at the control size's height;
    /// the accent ring while it has the caret.
    #[default]
    Bezeled,
    /// No box, no padding and no ground of its own: font, size, weight, tracking and colour are
    /// the parent's, so a property row's value, a title or a row's name is edited where it reads,
    /// and a container that draws the field's ground shows the focus. Only the caret
    /// (`--accent`) and the selection are styled.
    Plain,
}

/// A value the field rejects, and which rejection it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invalid {
    /// What is wrong, drawn under the field in the danger ink.
    pub message: TextLine,
    /// Which rejection this is: a secure field shakes once for each new stamp and not again for
    /// the same one (design/26-DETAILS.md R6).
    pub stamp: EventStamp,
}

/// Whether the field's value is acceptable, `data-validity`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Validity {
    /// Nothing is wrong.
    #[default]
    Valid,
    /// The value is rejected; the message says why.
    Invalid(Invalid),
}

impl Validity {
    /// `data-validity`: written only while the value is rejected.
    pub(crate) fn attr(&self) -> Option<&'static str> {
        match self {
            Validity::Valid => None,
            Validity::Invalid(_) => Some("invalid"),
        }
    }

    /// The message under the field, when it is rejected.
    pub(crate) fn message(&self) -> Option<&TextLine> {
        match self {
            Validity::Valid => None,
            Validity::Invalid(invalid) => Some(&invalid.message),
        }
    }
}

impl Detailed for Validity {
    /// A new rejection is a Failure (a secure field shakes on it); a rejection that clears or a
    /// stamp that stays is nothing to play.
    fn moment(from: &Self, to: &Self) -> Moment {
        match (from, to) {
            (Validity::Valid, Validity::Invalid(_)) => Moment::Failure,
            (Validity::Invalid(before), Validity::Invalid(after))
                if before.stamp != after.stamp =>
            {
                Moment::Failure
            }
            (Validity::Invalid(_), Validity::Invalid(_))
            | (Validity::Invalid(_), Validity::Valid)
            | (Validity::Valid, Validity::Valid) => Moment::Rest,
        }
    }

    /// A field that opens already rejected shows it still.
    fn first(state: &Self) -> Moment {
        match state {
            Validity::Valid | Validity::Invalid(_) => Moment::Rest,
        }
    }
}

/// One masked character.
pub(crate) const MASK_DOT: char = '\u{2022}';

impl FieldKind {
    /// The `type` attribute of the `input`.
    pub(crate) fn input_type(self) -> &'static str {
        match self {
            FieldKind::Secure => "password",
            FieldKind::Plain | FieldKind::Search => "text",
        }
    }

    /// `data-kind`: written for every kind but plain, so a plain field's markup is as it was.
    pub(crate) fn data_kind(self) -> Option<&'static str> {
        match self {
            FieldKind::Plain => None,
            FieldKind::Secure | FieldKind::Search => Some(self.slug()),
        }
    }

    /// The dots drawn over a masked value; nothing for an unmasked kind or an empty value.
    pub(crate) fn mask(self, value: &str) -> Option<String> {
        match self {
            FieldKind::Secure if !value.is_empty() => {
                Some(value.chars().map(|_| MASK_DOT).collect())
            }
            FieldKind::Secure | FieldKind::Plain | FieldKind::Search => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FieldKind, Invalid, Validity};
    use crate::components::content::text_runs::TextLine;
    use ds_motion::detail::{detailed::moment_table, moment::Moment, stamp::EventStamp};

    #[test]
    fn only_a_secure_field_is_masked() {
        const CASES: &[(FieldKind, &str, Option<&str>)] = &[
            (FieldKind::Plain, "abc", None),
            (FieldKind::Search, "abc", None),
            (FieldKind::Secure, "ab", Some("\u{2022}\u{2022}")),
            (FieldKind::Secure, "", None),
        ];
        for &(kind, value, want) in CASES {
            assert_eq!(kind.mask(value).as_deref(), want, "{kind:?} {value:?}");
        }
    }

    fn invalid(stamp: u32) -> Validity {
        Validity::Invalid(Invalid {
            message: TextLine::from("Wrong password"),
            stamp: EventStamp(stamp),
        })
    }

    #[test]
    fn a_new_rejection_is_a_failure_and_the_same_one_is_nothing() {
        moment_table(&[
            (Validity::Valid, invalid(1), Moment::Failure),
            (invalid(1), invalid(2), Moment::Failure),
            (invalid(2), invalid(2), Moment::Rest),
            (invalid(2), Validity::Valid, Moment::Rest),
            (Validity::Valid, Validity::Valid, Moment::Rest),
        ]);
    }
}
