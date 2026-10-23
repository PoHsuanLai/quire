//! A key whose value is a choice among things somebody installed (design/22-SETTINGS.md section
//! 9.1, `KeyKind::Provided`): the stored word `standard`, or the id of the thing chosen.
//!
//! This crate knows nothing about what is installed. The schema says only which kind of thing the
//! key picks (`capability = "look"`); the caller that draws the page lists the installed choices
//! and hands them to [`KeySpec::offering`](super::KeySpec::offering), and the program that reads
//! the key checks the stored id against what it found. A key that stores an id nobody has reads as
//! `standard` there, and the caller reports it once.

use serde::{Deserialize, Serialize};

/// The stored word for "the built-in one".
pub const STANDARD: &str = "standard";

/// A provided key's stored value: [`STANDARD`], or the id of the thing the person chose.
///
/// Any text reads, so a hand-written file never loses its other keys; whether the id is installed
/// is for the caller that knows what is installed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Provided(String);

impl Provided {
    /// The built-in choice.
    pub fn standard() -> Provided {
        Provided(STANDARD.to_owned())
    }

    /// The choice written as `text`: blank text is the built-in one.
    pub fn new(text: &str) -> Provided {
        let text = text.trim();
        if text.is_empty() {
            Provided::standard()
        } else {
            Provided(text.to_owned())
        }
    }

    /// The stored text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The id chosen, or `None` for the built-in choice (including blank stored text).
    pub fn chosen(&self) -> Option<&str> {
        let text = self.0.trim();
        (!text.is_empty() && text != STANDARD).then_some(text)
    }
}

impl Default for Provided {
    fn default() -> Provided {
        Provided::standard()
    }
}

#[cfg(test)]
mod tests {
    use super::{Provided, STANDARD};

    #[test]
    fn the_built_in_choice_is_standard_and_names_nothing() {
        let cases = ["", "  ", STANDARD];
        for text in cases {
            assert_eq!(Provided::new(text), Provided::standard(), "{text:?}");
            assert_eq!(Provided(text.to_owned()).chosen(), None, "{text:?}");
        }
        assert_eq!(Provided::default().as_str(), "standard");
    }

    #[test]
    fn anything_else_is_the_id_chosen() {
        assert_eq!(
            Provided::new(" org.example.Paper ").chosen(),
            Some("org.example.Paper")
        );
    }

    #[test]
    fn it_is_stored_as_bare_text() {
        assert_eq!(
            toml::Value::try_from(Provided::new("org.example.Paper")).ok(),
            Some(toml::Value::String("org.example.Paper".to_owned()))
        );
    }
}
