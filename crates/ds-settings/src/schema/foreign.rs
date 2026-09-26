//! [`SchemaVariants`] for `ds`'s own appearance enums.
//!
//! `ds-settings-derive`'s enum branch can only be attached to a type this crate defines — a
//! foreign type such as `ds::Theme` gets no `#[derive(SettingsSchema)]` at its own definition
//! (`crates/ds/src/appearance/**` is not a file this wave owns). The orphan rule still lets
//! this crate implement its own local trait for that foreign type, so the impl lives here
//! instead, read off each enum's existing `ALL` array through its `Serialize` — the same
//! lower-case word `appearance.toml` itself stores — rather than duplicating the slug spelling
//! by hand.

use serde::Serialize;

use super::traits::SchemaVariants;

/// Every member of `all`, as the word its `Serialize` (`#[serde(rename_all = "snake_case")]`)
/// writes. Panics only if a caller passes something that is not a bare unit variant, which
/// none of `ds`'s appearance enums are.
fn variants_of<T: Copy + Serialize>(all: &[T]) -> Vec<String> {
    all.iter()
        .map(|variant| match toml::Value::try_from(variant) {
            Ok(toml::Value::String(word)) => word,
            other => panic!("SchemaVariants: expected a bare enum word, got {other:?}"),
        })
        .collect()
}

impl SchemaVariants for ds::Theme {
    fn variants() -> Vec<String> {
        variants_of(&ds::Theme::ALL)
    }
}

impl SchemaVariants for ds::Look {
    fn variants() -> Vec<String> {
        variants_of(&ds::Look::ALL)
    }
}

impl SchemaVariants for ds::Warmth {
    fn variants() -> Vec<String> {
        variants_of(&ds::Warmth::ALL)
    }
}

impl SchemaVariants for ds::Accent {
    fn variants() -> Vec<String> {
        variants_of(&ds::Accent::ALL)
    }
}

impl SchemaVariants for ds::Motion {
    fn variants() -> Vec<String> {
        variants_of(&ds::Motion::ALL)
    }
}

impl SchemaVariants for ds::Typeface {
    fn variants() -> Vec<String> {
        variants_of(&ds::Typeface::ALL)
    }
}

#[cfg(test)]
mod tests {
    use super::SchemaVariants;

    #[test]
    fn theme_is_three_words_matching_its_own_serde() {
        assert_eq!(
            ds::Theme::variants(),
            vec!["system".to_owned(), "light".to_owned(), "dark".to_owned()]
        );
    }

    #[test]
    fn accent_is_six_words() {
        assert_eq!(ds::Accent::variants().len(), 6);
        assert_eq!(ds::Accent::variants()[0], "postmark");
    }

    #[test]
    fn motion_is_five_words() {
        assert_eq!(
            ds::Motion::variants(),
            vec![
                "system".to_owned(),
                "calm".to_owned(),
                "standard".to_owned(),
                "extra".to_owned(),
                "reduced".to_owned(),
            ]
        );
    }

    #[test]
    fn typeface_is_two_words() {
        assert_eq!(
            ds::Typeface::variants(),
            vec!["system".to_owned(), "editorial".to_owned()]
        );
    }

    #[test]
    fn look_is_four_words() {
        assert_eq!(ds::Look::variants().len(), 4);
    }

    #[test]
    fn warmth_is_four_words() {
        assert_eq!(ds::Warmth::variants().len(), 4);
    }
}
