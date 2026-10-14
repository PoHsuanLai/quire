//! A change the person made, and the values it reads and writes.

use ds_settings::schema::{KeyPath, KeySpec};

/// A change the person asked for: the key's path (`domain.key`) and the value it takes. The page
/// never saves; the app applies this to its values and writes them through its own `Store`.
#[derive(Debug, Clone, PartialEq)]
pub struct SettingEdit {
    pub path: KeyPath,
    pub value: toml::Value,
}

impl SettingEdit {
    /// `values` with this edit applied, the tables on the way made where missing. Values that are
    /// not a table are returned unchanged.
    pub fn applied_to(&self, mut values: toml::Value) -> toml::Value {
        let parts: Vec<&str> = self.path.0.split('.').collect();
        let mut at = &mut values;
        for (index, part) in parts.iter().enumerate() {
            let Some(table) = at.as_table_mut() else {
                return values;
            };
            if index + 1 == parts.len() {
                table.insert((*part).to_owned(), self.value.clone());
                return values;
            }
            at = table
                .entry((*part).to_owned())
                .or_insert_with(|| toml::Value::Table(toml::Table::new()));
        }
        values
    }
}

/// The value at `key`'s path in `values`, else the key's default.
pub(crate) fn value_of(values: &toml::Value, key: &KeySpec) -> toml::Value {
    key.path
        .0
        .split('.')
        .try_fold(values, |at, part| at.get(part))
        .cloned()
        .unwrap_or_else(|| key.default.clone())
}

#[cfg(test)]
#[path = "edit_tests.rs"]
mod tests;
