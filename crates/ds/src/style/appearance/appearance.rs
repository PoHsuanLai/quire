//! How a surface looks, as data: the three choices a `.ds` root resolves (design/04-COMPONENTS.md
//! section 26, O-16: Theme, Accent, Motion).
//!
//! Loading is lenient: a missing field is the first-run value and an unknown word for one field
//! is that field's default, never a failure of the whole value.

use crate::style::appearance::{accent::Accent, motion::Motion, theme::Theme};
use ds_core::word::Word;
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};

/// How the window looks, as data.
///
/// A missing field is the first-run value. An unknown word for one field is that field's
/// default, not a failure of the whole value. An unknown field is ignored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default)]
#[serde(default)]
pub struct Appearance {
    /// Which palette the window resolves to.
    #[serde(deserialize_with = "de_theme")]
    pub theme: Theme,
    /// The card's accent.
    #[serde(deserialize_with = "de_accent")]
    pub accent: Accent,
    /// How much the window moves.
    #[serde(deserialize_with = "de_motion")]
    pub motion: Motion,
}

/// A stored theme. Anything that is not `system`, `light` or `dark` is [`Theme::default`].
fn de_theme<'de, D>(deserializer: D) -> Result<Theme, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(word(deserializer)?
        .and_then(|word| Theme::parse(&word))
        .unwrap_or_default())
}

/// A stored accent. Anything that is not one of the six is [`Accent::default`].
fn de_accent<'de, D>(deserializer: D) -> Result<Accent, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(word(deserializer)?
        .and_then(|word| Accent::parse(&word))
        .unwrap_or_default())
}

/// A stored motion level. Anything that is not one of the five is [`Motion::default`].
fn de_motion<'de, D>(deserializer: D) -> Result<Motion, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(word(deserializer)?
        .and_then(|word| Motion::parse(&word))
        .unwrap_or_default())
}

/// The stored value as a word, or `None` when it is some other shape (a number, a table), so
/// `"accent": 7` costs only the accent.
fn word<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Stored {
        Word(String),
        Other(serde::de::IgnoredAny),
    }
    Ok(match Stored::deserialize(deserializer)? {
        Stored::Word(word) => Some(word),
        Stored::Other(_) => None,
    })
}

#[cfg(test)]
mod tests {
    use super::Appearance;
    use crate::style::appearance::{accent::Accent, motion::Motion, theme::Theme};

    #[test]
    fn a_partial_value_keeps_the_fields_it_has() {
        let cases: &[(&str, &str, Appearance)] = &[
            (
                "theme only",
                r#"{"theme":"dark"}"#,
                Appearance {
                    theme: Theme::Dark,
                    ..Appearance::default()
                },
            ),
            (
                "unknown theme keeps the motion",
                r#"{"theme":"sepia","motion":"reduced"}"#,
                Appearance {
                    motion: Motion::Reduced,
                    ..Appearance::default()
                },
            ),
            (
                "unknown motion keeps the theme",
                r#"{"theme":"dark","motion":"wild"}"#,
                Appearance {
                    theme: Theme::Dark,
                    ..Appearance::default()
                },
            ),
            (
                "an accent that is not a word keeps the motion",
                r#"{"accent":7,"motion":"reduced"}"#,
                Appearance {
                    motion: Motion::Reduced,
                    ..Appearance::default()
                },
            ),
            (
                "every field",
                r#"{"theme":"dark","accent":"violet","motion":"reduced","future":true}"#,
                Appearance {
                    theme: Theme::Dark,
                    accent: Accent::Violet,
                    motion: Motion::Reduced,
                },
            ),
        ];
        for &(name, json, want) in cases {
            let got: Appearance =
                serde_json::from_str(json).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(got, want, "{name}: {json}");
        }
    }
}
