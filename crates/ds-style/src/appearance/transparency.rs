//! Whether translucency is allowed: the preference a person picks ([`Transparency`]), macOS's
//! "Reduce transparency" (design/27-HIG-PARITY.md section 3.1).
//!
//! The settings portal has no key for it, so it is ours: `appearance.transparency`, one
//! desktop-wide setting that the compositor reads from the same appearance file as `theme`.

use ds_core::word::Word;
use serde::{Deserialize, Serialize};

/// Whether materials may show what is behind them, as a person chooses it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default, Word)]
#[serde(rename_all = "snake_case")]
pub enum Transparency {
    /// Materials are translucent wherever the compositor can blur behind them.
    #[default]
    System,
    /// Every material paints its solid tint (alpha at least .94) and the vibrancy boost is off,
    /// as when the compositor cannot blur.
    Reduced,
}

impl Transparency {
    /// The `data-transparency` value a root carries: only `Reduced` writes one, so a root that
    /// keeps translucency is marked as it always was.
    pub const fn attribute(self) -> Option<&'static str> {
        match self {
            Transparency::System => None,
            Transparency::Reduced => Some("reduced"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Transparency;
    use ds_core::word::Word;

    #[test]
    fn words_attributes_and_the_default_agree() {
        const CASES: &[(&str, Transparency, &str, Option<&str>)] = &[
            ("system", Transparency::System, "system", None),
            ("reduced", Transparency::Reduced, "reduced", Some("reduced")),
        ];
        assert_eq!(Transparency::default(), Transparency::System);
        for &(name, value, word, attribute) in CASES {
            assert_eq!(value.slug(), word, "{name}");
            assert_eq!(Transparency::parse(word), Some(value), "{name}");
            assert_eq!(value.attribute(), attribute, "{name}");
        }
        assert_eq!(Transparency::parse("translucent"), None);
    }
}
