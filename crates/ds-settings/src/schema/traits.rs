//! The two seams `#[derive(SettingsSchema)]` codes against (design/22-SETTINGS.md section 9.1):
//! [`SettingsSchema`] on a settings struct, [`SchemaVariants`] on the enum one of its fields
//! holds.

use serde::Serialize;

use super::key::{KeyKind, kind_from_variants};
use super::program::Schema;

/// A settings domain struct: `AppearanceSettings`, `IconsSettings`, .... Implemented by
/// `#[derive(SettingsSchema)]`, never by hand — `crates/ds-settings-derive`.
pub trait SettingsSchema {
    /// Every key this struct's fields describe, built from `Self::default()` and each field's
    /// `#[settings(...)]` attributes.
    fn schema() -> Schema;
}

/// A closed, fieldless enum used as a settings value: `Theme`, `Look`, `PlateGlyphPolicy`, ....
/// Implemented by `#[derive(SettingsSchema)]` on the enum itself for a type this crate owns
/// (`crates/ds-settings-derive`), or by hand for a foreign type such as `ds::Theme`
/// (`crate::schema::foreign`), which this crate may implement a local trait for but may not
/// add a derive to.
///
/// A field whose type implements neither this nor a shape the derive knows by name (text, a
/// colour, a list, a number with `range`) lands here; the message says what to add.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a settings enum, so the settings derive cannot pick its widget",
    label = "a settings field of this type needs a shape",
    note = "a number needs #[settings(range = \"min..=max\")]; a text newtype needs \
            #[settings(text)]; an enum needs #[derive(SettingsSchema)] (or a hand-written \
            SchemaVariants for a foreign type)"
)]
pub trait SchemaVariants {
    /// Every variant's stored word, in declaration order.
    fn variants() -> Vec<String>;
}

/// The `KeyKind` for a field whose type is `T`, from `T`'s own variant words
/// (design/22-SETTINGS.md section 9.1's widget-by-arity rule, `super::key::kind_from_variants`).
/// The struct derive's only call into `SchemaVariants` for an enum-shaped field.
pub fn kind_of<T: SchemaVariants>() -> KeyKind {
    kind_from_variants(T::variants())
}

/// `value` as the `toml::Value` a `KeySpec::default` holds.
///
/// # Panics
/// Only if `T`'s `Serialize` cannot reach TOML at all (a map with non-string keys, `NaN`, ...),
/// which none of this workspace's settings types can produce (`CONVENTIONS.md#2-derives`:
/// "floats are not allowed in domain types" keeps `Scalar`'s `f32` finite by construction).
pub fn to_value<T: Serialize>(value: &T) -> toml::Value {
    toml::Value::try_from(value)
        .expect("a settings field's value always serialises to a TOML value")
}

#[cfg(test)]
mod tests {
    use super::{SchemaVariants, kind_of};
    use crate::schema::KeyKind;

    struct TwoWords;
    impl SchemaVariants for TwoWords {
        fn variants() -> Vec<String> {
            vec!["a".to_owned(), "b".to_owned()]
        }
    }

    #[test]
    fn kind_of_asks_schema_variants() {
        assert!(matches!(kind_of::<TwoWords>(), KeyKind::Toggle { .. }));
    }
}
