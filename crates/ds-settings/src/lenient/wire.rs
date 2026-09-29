//! The two text formats a settings file is written in, seen as key trees: the lenient reader
//! lays each key of a file over the type's default one at a time, and asks the tree whether a
//! key survived a round trip through the type.

use serde::Serialize;
use serde::de::DeserializeOwned;

/// One format's tree of keys.
pub(super) trait Wire {
    /// A table of keys (`toml::Table`, a JSON object).
    type Table: Clone;
    /// One value in a table.
    type Value: Clone;

    /// `text` as a table; the reason when it is not this format at all.
    fn parse(text: &str) -> Result<Self::Table, String>;
    /// `value` as a table, or `None` when it has no table form.
    fn encode<D: Serialize>(value: &D) -> Option<Self::Table>;
    /// A `D` from `table`, or why it is not one.
    fn decode<D: DeserializeOwned>(table: &Self::Table) -> Result<D, String>;
    /// Every value in `table` that is not a non-empty table, with its key path, depth first.
    fn leaves(table: &Self::Table) -> Vec<(Vec<String>, Self::Value)>;
    /// Put `value` at `path`, replacing a non-table on the way with a table.
    fn set(table: &mut Self::Table, path: &[String], value: Self::Value);
    /// Whether `path` names a key in `table`.
    fn has(table: &Self::Table, path: &[String]) -> bool;
}

/// TOML, the format of every hand-edited file.
pub(super) struct TomlWire;

impl Wire for TomlWire {
    type Table = toml::Table;
    type Value = toml::Value;

    fn parse(text: &str) -> Result<toml::Table, String> {
        text.parse::<toml::Table>().map_err(|e| e.to_string())
    }

    fn encode<D: Serialize>(value: &D) -> Option<toml::Table> {
        match toml::Value::try_from(value) {
            Ok(toml::Value::Table(table)) => Some(table),
            _ => None,
        }
    }

    fn decode<D: DeserializeOwned>(table: &toml::Table) -> Result<D, String> {
        toml::Value::Table(table.clone())
            .try_into()
            .map_err(|e: toml::de::Error| e.message().to_owned())
    }

    fn leaves(table: &toml::Table) -> Vec<(Vec<String>, toml::Value)> {
        fn walk(table: &toml::Table, prefix: &[String]) -> Vec<(Vec<String>, toml::Value)> {
            table
                .iter()
                .flat_map(|(key, value)| {
                    let path: Vec<String> = prefix.iter().cloned().chain([key.clone()]).collect();
                    match value {
                        toml::Value::Table(inner) if !inner.is_empty() => walk(inner, &path),
                        other => vec![(path, other.clone())],
                    }
                })
                .collect()
        }
        walk(table, &[])
    }

    fn set(table: &mut toml::Table, path: &[String], value: toml::Value) {
        match path {
            [] => {}
            [last] => {
                table.insert(last.clone(), value);
            }
            [first, rest @ ..] => {
                let entry = table
                    .entry(first.clone())
                    .or_insert_with(|| toml::Value::Table(toml::Table::new()));
                if !entry.is_table() {
                    *entry = toml::Value::Table(toml::Table::new());
                }
                if let toml::Value::Table(inner) = entry {
                    Self::set(inner, rest, value);
                }
            }
        }
    }

    fn has(table: &toml::Table, path: &[String]) -> bool {
        match path {
            [] => true,
            [first, rest @ ..] => match table.get(first) {
                Some(toml::Value::Table(inner)) => Self::has(inner, rest),
                Some(_) => rest.is_empty(),
                None => false,
            },
        }
    }
}

/// JSON, the format of machine-written stores.
pub(super) struct JsonWire;

type Object = serde_json::Map<String, serde_json::Value>;

impl Wire for JsonWire {
    type Table = Object;
    type Value = serde_json::Value;

    fn parse(text: &str) -> Result<Object, String> {
        match serde_json::from_str::<serde_json::Value>(text) {
            Ok(serde_json::Value::Object(object)) => Ok(object),
            Ok(_) => Err("not a JSON object".to_owned()),
            Err(e) => Err(e.to_string()),
        }
    }

    fn encode<D: Serialize>(value: &D) -> Option<Object> {
        match serde_json::to_value(value) {
            Ok(serde_json::Value::Object(object)) => Some(object),
            _ => None,
        }
    }

    fn decode<D: DeserializeOwned>(table: &Object) -> Result<D, String> {
        serde_json::from_value(serde_json::Value::Object(table.clone())).map_err(|e| e.to_string())
    }

    fn leaves(table: &Object) -> Vec<(Vec<String>, serde_json::Value)> {
        fn walk(object: &Object, prefix: &[String]) -> Vec<(Vec<String>, serde_json::Value)> {
            object
                .iter()
                .flat_map(|(key, value)| {
                    let path: Vec<String> = prefix.iter().cloned().chain([key.clone()]).collect();
                    match value {
                        serde_json::Value::Object(inner) if !inner.is_empty() => walk(inner, &path),
                        other => vec![(path, other.clone())],
                    }
                })
                .collect()
        }
        walk(table, &[])
    }

    fn set(table: &mut Object, path: &[String], value: serde_json::Value) {
        match path {
            [] => {}
            [last] => {
                table.insert(last.clone(), value);
            }
            [first, rest @ ..] => {
                let entry = table
                    .entry(first.clone())
                    .or_insert_with(|| serde_json::Value::Object(Object::new()));
                if !entry.is_object() {
                    *entry = serde_json::Value::Object(Object::new());
                }
                if let serde_json::Value::Object(inner) = entry {
                    Self::set(inner, rest, value);
                }
            }
        }
    }

    fn has(table: &Object, path: &[String]) -> bool {
        match path {
            [] => true,
            [first, rest @ ..] => match table.get(first) {
                Some(serde_json::Value::Object(inner)) => Self::has(inner, rest),
                Some(_) => rest.is_empty(),
                None => false,
            },
        }
    }
}
