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
