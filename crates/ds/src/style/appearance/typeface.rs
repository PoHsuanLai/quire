//! Which voice a surface's type speaks in (design/02-TYPE.md section 2): the desktop's system
//! face, or mail's editorial faces.
//!
//! The root writes it as `data-typeface` on `.ds`, like the theme; the stylesheet's
//! `.ds[data-typeface=editorial]` block maps the family tokens back to the editorial faces.

use serde::{Deserialize, Serialize};

/// Which faces `--font-display`, `--font-ui` and `--font-data` name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum Typeface {
    /// The desktop's face: Inter for reading, controls and data (tabular), Inter Display for
    /// headings and big numbers.
    #[default]
    System,
    /// mail's voice: Bricolage Grotesque, Karla and Space Mono, as the mail prototype sets them.
    Editorial,
}

impl Typeface {
    /// Every choice, in the order a picker offers them.
    pub const ALL: [Typeface; 2] = [Typeface::System, Typeface::Editorial];

    /// The `data-typeface` value on a `.ds` root, and the word `appearance.toml` stores.
    pub fn slug(self) -> &'static str {
        match self {
            Typeface::System => "system",
            Typeface::Editorial => "editorial",
        }
    }

    /// What a picker calls it.
    pub fn label(self) -> &'static str {
        match self {
            Typeface::System => "System",
            Typeface::Editorial => "Editorial",
        }
    }

    /// The typeface a stored word names, or [`None`] for a word that is not one.
    pub fn parse(word: &str) -> Option<Typeface> {
        Typeface::ALL
            .into_iter()
            .find(|typeface| typeface.slug() == word)
    }
}

#[cfg(test)]
mod tests {
    use super::Typeface;

    #[test]
    fn the_slug_is_the_stored_word() {
        for typeface in Typeface::ALL {
            let json = serde_json::to_string(&typeface).unwrap_or_default();
            assert_eq!(json, format!("\"{}\"", typeface.slug()));
            assert_eq!(Typeface::parse(typeface.slug()), Some(typeface));
            let back: Typeface = serde_json::from_str(&json).unwrap_or_default();
            assert_eq!(back, typeface);
        }
        assert_eq!(Typeface::parse("serif"), None);
        assert_eq!(Typeface::default(), Typeface::System);
    }
}
