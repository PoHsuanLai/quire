//! How a surface looks, as data: the choices a `.ds` root resolves (design/04-COMPONENTS.md
//! section 26, O-16: Theme, Accent, Motion; and `appearance.transparency`).
//!
//! Loading is lenient: a missing field is the first-run value and an unknown word for one field
//! is that field's default, never a failure of the whole value.

use crate::appearance::{accent::Accent, motion::Motion, theme::Theme, transparency::Transparency};
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
    /// Whether materials may be translucent (`appearance.transparency`); written only when it
    /// is `Reduced`, so a file from before the field reads and writes as it did.
    #[serde(
        deserialize_with = "de_transparency",
        skip_serializing_if = "is_system"
    )]
    pub transparency: Transparency,
}

fn is_system(transparency: &Transparency) -> bool {
    *transparency == Transparency::System
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

/// A stored accent. Anything that is not one of the eight is [`Accent::default`].
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

/// A stored transparency. Anything that is not `system` or `reduced` is
/// [`Transparency::default`].
fn de_transparency<'de, D>(deserializer: D) -> Result<Transparency, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(word(deserializer)?
        .and_then(|word| Transparency::parse(&word))
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
    use crate::appearance::{
        accent::Accent, motion::Motion, theme::Theme, transparency::Transparency,
    };

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
                r#"{"theme":"dark","accent":"purple","motion":"reduced","future":true}"#,
                Appearance {
                    theme: Theme::Dark,
                    accent: Accent::Purple,
                    motion: Motion::Reduced,
                    ..Appearance::default()
                },
            ),
            (
                "an old file without the field is System",
                r#"{"theme":"dark","accent":"purple","motion":"reduced"}"#,
                Appearance {
                    theme: Theme::Dark,
                    accent: Accent::Purple,
                    motion: Motion::Reduced,
                    transparency: Transparency::System,
                },
            ),
            (
                "reduced transparency",
                r#"{"transparency":"reduced"}"#,
                Appearance {
                    transparency: Transparency::Reduced,
                    ..Appearance::default()
                },
            ),
            (
                "an unknown transparency keeps the theme",
                r#"{"theme":"dark","transparency":"clear"}"#,
                Appearance {
                    theme: Theme::Dark,
                    ..Appearance::default()
                },
            ),
            (
                "a transparency that is not a word keeps the motion",
                r#"{"transparency":1,"motion":"reduced"}"#,
                Appearance {
                    motion: Motion::Reduced,
                    ..Appearance::default()
                },
            ),
        ];
        for &(name, json, want) in cases {
            let got: Appearance =
                serde_json::from_str(json).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(got, want, "{name}: {json}");
        }
    }

    #[test]
    fn only_reduced_transparency_is_written() {
        let reduced = Appearance {
            transparency: Transparency::Reduced,
            ..Appearance::default()
        };
        let cases = [
            (
                Appearance::default(),
                r#"{"theme":"system","accent":"blue","motion":"standard"}"#,
            ),
            (
                reduced,
                r#"{"theme":"system","accent":"blue","motion":"standard","transparency":"reduced"}"#,
            ),
        ];
        for (value, want) in cases {
            assert_eq!(serde_json::to_string(&value).unwrap_or_default(), want);
        }
    }
}
