//! How a surface looks, as data: the three choices a `.ds` root resolves (design/04-COMPONENTS.md
//! section 26, O-16: Theme, Accent, Motion).
//!
//! Moved from mailo (`mail-app/src/view.rs`). mailo's `marks` (provider icons or letters) is a
//! mail preference and stays in mailo; `accent` came back as one of six. Loading is lenient
//! exactly as mailo's was: a missing field is the first-run value and an unknown word for one
//! field is that field's default, never a failure of the whole value.

use super::{Accent, Motion, Theme};
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

/// The stored value as a word, or `None` when it is some other shape (a number, a table).
///
/// mailo read `String::deserialize` here, which failed the whole file on `"accent": 7`; its
/// own migration test only passed because the retired field was skipped before this ran.
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
    use crate::appearance::{Accent, Motion, Theme};

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
                r#"{"theme":"sepia","motion":"calm"}"#,
                Appearance {
                    motion: Motion::Calm,
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
                "a retired accent word keeps the theme",
                r#"{"accent":"pine","theme":"light"}"#,
                Appearance {
                    theme: Theme::Light,
                    ..Appearance::default()
                },
            ),
            (
                "an accent that is not a word keeps the motion",
                r#"{"accent":7,"motion":"extra"}"#,
                Appearance {
                    motion: Motion::Extra,
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
