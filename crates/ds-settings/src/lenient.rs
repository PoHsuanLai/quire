//! The lenient reader every settings file uses (design/22-SETTINGS.md section 2, "Unknown
//! values fall back to the field's default"): a bad value costs that one key, never the file.
//!
//! The doc sketches this as a `deserialize_with` helper that falls back to the field *type's*
//! `Default`; that would turn a bad `material_tint_alpha` into 0 rather than the key's shipped
//! 80. So the fallback is the *struct's* default for that key: start from `T::default()` as a
//! TOML tree, lay each key the file sets over it one at a time, and keep a key only if `T`
//! still deserializes with it (FINDINGS.md).

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Parse `text` as `T`, keeping every key that is valid and the default for every key that is
/// not. Text that is not TOML at all is `T::default()`.
pub fn lenient<T>(text: &str) -> T
where
    T: Serialize + DeserializeOwned + Default,
{
    let Ok(given) = text.parse::<toml::Table>() else {
        return T::default();
    };
    let Ok(toml::Value::Table(mut tree)) = toml::Value::try_from(T::default()) else {
        return T::default();
    };
    for (path, value) in leaves(&given, Vec::new()) {
        let mut candidate = tree.clone();
        set(&mut candidate, &path, value);
        if toml::Value::Table(candidate.clone())
            .try_into::<T>()
            .is_ok()
        {
            tree = candidate;
        }
    }
    toml::Value::Table(tree).try_into().unwrap_or_default()
}

/// Every non-table value in `table` with its key path, depth first.
fn leaves(table: &toml::Table, prefix: Vec<String>) -> Vec<(Vec<String>, toml::Value)> {
    table
        .iter()
        .flat_map(|(key, value)| {
            let mut path = prefix.clone();
            path.push(key.clone());
            match value {
                toml::Value::Table(inner) if !inner.is_empty() => leaves(inner, path),
                other => vec![(path, other.clone())],
            }
        })
        .collect()
}

/// Put `value` at `path` in `tree`, replacing a non-table on the way with a table.
fn set(tree: &mut toml::Table, path: &[String], value: toml::Value) {
    match path {
        [] => {}
        [last] => {
            tree.insert(last.clone(), value);
        }
        [first, rest @ ..] => {
            let entry = tree
                .entry(first.clone())
                .or_insert_with(|| toml::Value::Table(toml::Table::new()));
            if !entry.is_table() {
                *entry = toml::Value::Table(toml::Table::new());
            }
            if let toml::Value::Table(inner) = entry {
                set(inner, rest, value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::lenient;
    use crate::settings::AppearanceFile;
    use crate::units::Percent;
    use ds::{Motion, Theme};

    #[test]
    fn text_that_is_not_toml_at_all_is_the_default() {
        const CASES: &[&str] = &["", "not toml [[[ = =", "\u{0}\u{1}garbage"];
        for text in CASES {
            assert_eq!(
                lenient::<AppearanceFile>(text),
                AppearanceFile::default(),
                "{text:?}"
            );
        }
    }

    #[test]
    fn an_unknown_enum_word_falls_back_to_that_fields_default_and_keeps_its_siblings() {
        let file: AppearanceFile =
            lenient("[appearance]\ntheme = \"sepia\"\nmotion_level = \"calm\"\n");
        assert_eq!(
            file.appearance.theme,
            Theme::default(),
            "bad field defaults"
        );
        assert_eq!(
            file.appearance.motion_level,
            Motion::Calm,
            "sibling field is kept"
        );
    }

    #[test]
    fn an_out_of_range_percent_is_clamped_not_defaulted() {
        let file: AppearanceFile = lenient("[icons]\nplate_inset_percent = 255\n");
        assert_eq!(file.icons.plate_inset_percent, Percent(100));
    }

    #[test]
    fn a_missing_table_is_that_tables_defaults() {
        let file: AppearanceFile = lenient("[appearance]\ntheme = \"dark\"\n");
        assert_eq!(file.appearance.theme, Theme::Dark);
        assert_eq!(file.icons, crate::settings::IconsSettings::default());
    }
}
