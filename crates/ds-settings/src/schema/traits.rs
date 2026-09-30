//! The seam `#[derive(SettingsSchema)]` codes against (design/22-SETTINGS.md section 9.1): the
//! [`SettingsSchema`] a settings struct implements, and [`kind_of`], which reads a closed enum
//! field's variant words off its own [`Word`] vocabulary.

use ds_core::word::Word;
use serde::Serialize;

use super::key::{KeyKind, kind_from_variants};
use super::program::Schema;

/// A settings domain struct: `AppearanceSettings`, `IconsSettings`, .... Implemented by
/// `#[derive(SettingsSchema)]`, never by hand (`crates/ds-settings-derive`).
pub trait SettingsSchema {
    /// Every key this struct's fields describe, built from `Self::default()` and each field's
    /// `#[settings(...)]` attributes.
    fn schema() -> Schema;
}

/// The words a stored [`Word`] enum is written as, in declaration order: the same words its
/// `Serialize` puts in the file, so a schema and a file always agree on the spelling.
///
/// # Panics
/// Only if a variant serialises as something other than a bare word, which a fieldless enum
/// never does.
fn stored_words<T: Word + Serialize>() -> Vec<String> {
    T::ALL
        .iter()
        .map(|variant| match toml::Value::try_from(variant) {
            Ok(toml::Value::String(word)) => word,
            other => panic!("a settings enum stores bare words, got {other:?}"),
        })
        .collect()
}

/// The `KeyKind` for a field whose type is the closed enum `T` (design/22-SETTINGS.md section
/// 9.1's widget-by-arity rule, `super::key::kind_from_variants`). The struct derive's only call
/// for an enum-shaped field: a field type that is neither a shape the derive knows by name (text,
/// a colour, a list, a number with `range`) nor a `Word` fails to compile here.
pub fn kind_of<T: Word + Serialize>() -> KeyKind {
    kind_from_variants(stored_words::<T>())
}

/// `value` as the `toml::Value` a `KeySpec::default` holds.
///
/// # Panics
/// Only if `T`'s `Serialize` cannot reach TOML at all (a map with non-string keys, `NaN`, ...),
/// which none of this workspace's settings types can produce
/// (`CONVENTIONS.md#12-derives-and-serde`: floats stay out of data, and `Scalar`'s `f32` is finite
/// by construction).
pub fn to_value<T: Serialize>(value: &T) -> toml::Value {
    toml::Value::try_from(value)
        .expect("a settings field's value always serialises to a TOML value")
}

#[cfg(test)]
mod tests {
    use super::{kind_of, stored_words};
    use crate::schema::KeyKind;

    #[test]
    fn appearance_enums_are_the_words_their_own_serde_writes() {
        let cases: &[(&str, Vec<String>, &[&str])] = &[
            (
                "theme",
                stored_words::<ds::Theme>(),
                &["system", "light", "dark"],
            ),
            (
                "motion",
                stored_words::<ds::Motion>(),
                &["standard", "reduced"],
            ),
            (
                "typeface",
                stored_words::<ds::Typeface>(),
                &["system", "editorial"],
            ),
            ("look", stored_words::<ds::Look>(), &["mac"]),
        ];
        for (name, got, want) in cases {
            assert_eq!(got, want, "{name}");
        }
        assert_eq!(
            stored_words::<ds::Accent>().first().map(String::as_str),
            Some("postmark")
        );
        assert_eq!(stored_words::<ds::Accent>().len(), 6);
    }

    #[test]
    fn a_two_word_enum_is_a_toggle() {
        assert!(matches!(kind_of::<ds::Motion>(), KeyKind::Toggle { .. }));
    }
}
