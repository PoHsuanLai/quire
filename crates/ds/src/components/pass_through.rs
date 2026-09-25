//! What a consumer may add to a quire control's own element (mailo gaps 6): `data-*`
//! attributes of its own and classes of its own.
//!
//! mailo wrapped its buttons in a `span` to carry `data-folder` (its drag reads the drop place
//! off the element under the pointer, and its tests find a folder's button by it) and to hang a
//! class on for a hover reveal. A wrapper is a second box in the layout and a second target for
//! the pointer. Letting the button carry them removes both, and the types below keep the one
//! rule that matters: nothing a consumer adds can look like quire's own vocabulary. A name or a
//! class that starts `ds-` is refused when it is built, and so is a `data-*` name quire writes
//! itself (`data-variant`, `data-size`) or reads on the root (`data-theme`, `data-accent`,
//! `data-motion`, `data-material`), so no consumer attribute can restyle a quire element.

use dioxus::core::Attribute;
use std::collections::BTreeSet;
use std::fmt;
use std::sync::Mutex;

/// Why a pass-through name or class was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PassThroughError {
    /// Empty, or a character outside lowercase ASCII letters, digits and `-`, or not starting
    /// with a letter (a `data-*` name is lowercase in HTML: an uppercase one would be folded
    /// by a browser and kept by Blitz, so the two would disagree).
    #[error(
        "a data attribute name is lowercase letters, digits and '-', starting with a letter: {name:?}"
    )]
    BadDataName {
        /// The name as given.
        name: String,
    },
    /// A class token outside letters, digits, `-` and `_`, or not starting with a letter or
    /// `_`, or no token at all.
    #[error("a class is letters, digits, '-' and '_', starting with a letter or '_': {class:?}")]
    BadClass {
        /// The class list as given.
        class: String,
    },
    /// The name or a class starts `ds-`, or the name is one quire writes or reads itself.
    #[error("{given:?} is quire's own vocabulary; a consumer's name or class cannot use it")]
    Reserved {
        /// The name or class as given.
        given: String,
    },
}

/// `data-*` names quire writes on its controls or reads on the root: a consumer attribute of
/// the same name would change how quire's stylesheet draws the element.
const RESERVED_DATA: &[&str] = &["variant", "size", "theme", "accent", "motion", "material"];

/// The prefix of every quire class and of the names a consumer cannot take.
const DS_PREFIX: &str = "ds-";

/// A consumer's `data-*` attribute name, without the `data-` (`DataName::parse("folder")` is
/// written `data-folder`).
///
/// Names are a small closed vocabulary of the consumer's own (`folder`, `thread`), so each
/// distinct one is interned once, for the life of the program, as the `&'static str` the
/// renderer's attribute takes. The value is where the data goes; build names from constants,
/// never from data, or the set grows with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DataName {
    /// The whole attribute name, `data-` included.
    attribute: &'static str,
}

impl DataName {
    /// The name, checked: lowercase ASCII letters, digits and `-`, starting with a letter; not
    /// `ds-…`, and not a name quire uses itself.
    pub fn parse(name: &str) -> Result<Self, PassThroughError> {
        let shaped = name.starts_with(|c: char| c.is_ascii_lowercase())
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if !shaped {
            return Err(PassThroughError::BadDataName {
                name: name.to_string(),
            });
        }
        if name.starts_with(DS_PREFIX) || name == "ds" || RESERVED_DATA.contains(&name) {
            return Err(PassThroughError::Reserved {
                given: name.to_string(),
            });
        }
        Ok(DataName {
            attribute: intern(&format!("data-{name}")),
        })
    }

    /// The attribute as written: `data-folder`.
    pub fn attribute(self) -> &'static str {
        self.attribute
    }
}

impl fmt::Display for DataName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.attribute)
    }
}

/// The attribute names handed out so far: one leaked string per distinct name.
static NAMES: Mutex<BTreeSet<&'static str>> = Mutex::new(BTreeSet::new());

/// `attribute` as a `&'static str`, the same one every time it is asked for.
fn intern(attribute: &str) -> &'static str {
    // A poisoned lock still holds a valid set: the only writer inserts a finished string.
    let mut names = NAMES
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(known) = names.get(attribute) {
        return known;
    }
    let leaked: &'static str = Box::leak(attribute.to_string().into_boxed_str());
    names.insert(leaked);
    leaked
}

/// One `data-*` attribute a consumer puts on a quire control: `data-<name>="<value>"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataAttr {
    /// The checked name.
    pub name: DataName,
    /// The value, as the consumer words it (a folder's path).
    pub value: String,
}

impl DataAttr {
    /// An attribute from a checked name.
    pub fn new(name: DataName, value: impl Into<String>) -> Self {
        DataAttr {
            name,
            value: value.into(),
        }
    }
}

/// The attributes, for an element's spread.
pub(crate) fn attributes(data: &[DataAttr]) -> Vec<Attribute> {
    data.iter()
        .map(|attr| Attribute::new(attr.name.attribute(), attr.value.clone(), None, false))
        .collect()
}

/// A consumer's own class or classes on a quire control (`ExtraClass::parse("fold-more")`),
/// appended after quire's. For the consumer's own layout and reveal rules; it can never be a
/// `ds-` class, so it cannot reach into quire's rules (CONVENTIONS section 11).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExtraClass(String);

impl ExtraClass {
    /// One class, or a space-separated list, checked: each token letters, digits, `-` and
    /// `_`, starting with a letter or `_`, none starting `ds-`. Runs of whitespace fold to one
    /// space.
    pub fn parse(class: &str) -> Result<Self, PassThroughError> {
        let tokens: Vec<&str> = class.split_ascii_whitespace().collect();
        let refuse = || PassThroughError::BadClass {
            class: class.to_string(),
        };
        if tokens.is_empty() || !tokens.iter().all(|token| class_token(token)) {
            return Err(refuse());
        }
        if tokens.iter().any(|token| token.starts_with(DS_PREFIX)) {
            return Err(PassThroughError::Reserved {
                given: class.to_string(),
            });
        }
        Ok(ExtraClass(tokens.join(" ")))
    }

    /// The classes, space-separated.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ExtraClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Whether `token` is one class name of the shape [`ExtraClass`] takes.
fn class_token(token: &str) -> bool {
    token.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
        && token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// quire's class list with the consumer's appended: `ds-button fold-more`.
pub(crate) fn class_list(own: &str, extra: Option<&ExtraClass>) -> String {
    match extra {
        Some(extra) => format!("{own} {extra}"),
        None => own.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{DataName, ExtraClass, PassThroughError, class_list};

    #[test]
    fn a_data_name_is_a_lowercase_word_of_the_consumers() {
        let cases: &[(&str, Result<&str, ()>)] = &[
            ("folder", Ok("data-folder")),
            ("thread-id", Ok("data-thread-id")),
            ("x2", Ok("data-x2")),
            ("", Err(())),
            ("Folder", Err(())),
            ("2x", Err(())),
            ("-x", Err(())),
            ("fol der", Err(())),
            ("folder_id", Err(())),
            ("ds-drop", Err(())),
            ("ds", Err(())),
            ("variant", Err(())),
            ("theme", Err(())),
        ];
        for (given, want) in cases {
            let got = DataName::parse(given)
                .map(DataName::attribute)
                .map_err(|_| ());
            assert_eq!(got, *want, "{given:?}");
        }
    }

    #[test]
    fn the_same_name_is_the_same_string() {
        let one = DataName::parse("folder").map(DataName::attribute);
        let two = DataName::parse("folder").map(DataName::attribute);
        match (one, two) {
            (Ok(one), Ok(two)) => assert!(std::ptr::eq(one, two), "interned once"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn an_extra_class_is_the_consumers_and_never_a_ds_one() {
        let cases: &[(&str, Result<&str, ()>)] = &[
            ("fold-more", Ok("fold-more")),
            ("  fold-more   reveal ", Ok("fold-more reveal")),
            ("_private", Ok("_private")),
            ("", Err(())),
            ("   ", Err(())),
            ("1st", Err(())),
            ("a.b", Err(())),
            ("ds-button", Err(())),
            ("reveal ds-button", Err(())),
        ];
        for (given, want) in cases {
            let got = ExtraClass::parse(given);
            assert_eq!(
                got.as_ref().map(ExtraClass::as_str).map_err(|_| ()),
                *want,
                "{given:?}"
            );
        }
        assert_eq!(
            ExtraClass::parse("ds-x"),
            Err(PassThroughError::Reserved {
                given: "ds-x".to_string()
            })
        );
    }

    #[test]
    fn the_consumers_classes_follow_quires() {
        let extra = ExtraClass::parse("fold-more").ok();
        assert_eq!(
            class_list("ds-button", extra.as_ref()),
            "ds-button fold-more"
        );
        assert_eq!(class_list("ds-button", None), "ds-button");
    }
}
