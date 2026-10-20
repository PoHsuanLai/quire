//! Changing one key of a settings file in place.
//!
//! A program that does not own the file (the Settings app, an agent) or owns only a field of it
//! (a consent flag) must not load the whole document and save it back: a change made in between
//! would be lost. [`SettingsWriter::patch`] reads the file under its lock, sets the one key in
//! the parsed text and writes the result atomically, so every comment, every other key and the
//! person's formatting stay as they were.

use crate::doc::SettingsDoc;
use crate::error::SettingsError;
use crate::lenient::SETTINGS_CAP;
use crate::schema::KeyPath;
use crate::store::Store;
use ds_helpers::{NewFileMode, UpdateError, update_atomic};
use std::path::{Path, PathBuf};
use toml_edit::{DocumentMut, InlineTable, Item, Table, TableLike, Value};

/// The writer of one TOML settings file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsWriter {
    path: PathBuf,
}

impl SettingsWriter {
    /// A writer for the file at `path`. The file need not exist yet.
    pub fn new(path: PathBuf) -> SettingsWriter {
        SettingsWriter { path }
    }

    /// A writer for the file `D` is stored in inside `store`'s directory.
    pub fn of<D: SettingsDoc>(store: &Store) -> Result<SettingsWriter, SettingsError> {
        store
            .path::<D>()
            .map(SettingsWriter::new)
            .ok_or_else(|| store.no_config_dir())
    }

    /// The file this writes.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Sets `key` (a dotted path) to `value`, making the tables on the way, and leaves every
    /// other byte of the file as it was. The key keeps its place, spacing and trailing comment
    /// when it is already there. A table value is written as an inline table
    /// (`consent = { given = 1760000000 }`). The read, the edit and the write happen under the
    /// file's lock ([`ds_helpers::update_atomic`]): a crash cannot leave half a file and a
    /// directory watch sees one rename.
    ///
    /// A file that is there and cannot be used, or is not valid TOML, or a key under a value that
    /// is not a table, is left alone and reported. A directory that is not there is made.
    pub fn patch(&self, key: &KeyPath, value: &toml::Value) -> Result<(), SettingsError> {
        self.patch_all(&[(key.clone(), value.clone())])
    }

    /// Applies every edit, in order, under one lock and in one atomic write. If any edit fails
    /// nothing is written.
    pub fn patch_all(&self, edits: &[(KeyPath, toml::Value)]) -> Result<(), SettingsError> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(|source| SettingsError::Io {
                path: dir.to_path_buf(),
                source,
            })?;
        }
        let update = update_atomic(&self.path, SETTINGS_CAP, NewFileMode::Umask, |existing| {
            self.edited(existing.as_deref().unwrap_or_default(), edits)
        });
        match update {
            Ok(()) => Ok(()),
            Err(UpdateError::Read(error)) => Err(error.into()),
            Err(UpdateError::Edit(error)) => Err(error),
            Err(UpdateError::Write(error)) => Err(error.into()),
            Err(_) => Err(self.refuse("the file could not be updated".to_owned())),
        }
    }

    /// `text` with every edit applied.
    fn edited(
        &self,
        text: &str,
        edits: &[(KeyPath, toml::Value)],
    ) -> Result<String, SettingsError> {
        let mut doc: DocumentMut = text
            .parse()
            .map_err(|e: toml_edit::TomlError| self.refuse(e.message().to_owned()))?;
        for (key, value) in edits {
            let parts: Vec<&str> = key.0.split('.').collect();
            if parts.iter().any(|part| part.is_empty()) {
                return Err(self.refuse(format!("{:?} is not a key path", key.0)));
            }
            set(doc.as_table_mut(), &parts, to_edit(value)).map_err(|()| {
                self.refuse(format!("{} sits inside a value that is not a table", key.0))
            })?;
        }
        Ok(doc.to_string())
    }

    fn refuse(&self, reason: String) -> SettingsError {
        SettingsError::Patch {
            path: self.path.clone(),
            reason,
        }
    }
}

/// `value` as a document value; a table becomes an inline table.
fn to_edit(value: &toml::Value) -> Value {
    match value {
        toml::Value::String(text) => Value::from(text.as_str()),
        toml::Value::Integer(number) => Value::from(*number),
        toml::Value::Float(number) => Value::from(*number),
        toml::Value::Boolean(flag) => Value::from(*flag),
        toml::Value::Datetime(when) => when
            .to_string()
            .parse()
            .unwrap_or_else(|_| Value::from(when.to_string())),
        toml::Value::Array(items) => Value::Array(items.iter().map(to_edit).collect()),
        toml::Value::Table(table) => {
            let mut inline = InlineTable::new();
            for (name, item) in table {
                inline.insert(name, to_edit(item));
            }
            Value::InlineTable(inline)
        }
    }
}

/// Sets `path` in `table` (a real or inline table), making the tables on the way. The key stays
/// where it is, with its own spacing and comments; only the value is replaced, taking the old
/// value's spacing and trailing comment.
fn set(table: &mut dyn TableLike, path: &[&str], value: Value) -> Result<(), ()> {
    match path {
        [] => Ok(()),
        [last] => {
            match table.get_mut(last) {
                Some(item) => {
                    let mut value = value;
                    if let Some(old) = item.as_value() {
                        *value.decor_mut() = old.decor().clone();
                    }
                    *item = Item::Value(value);
                }
                None => {
                    table.insert(last, Item::Value(value));
                }
            }
            Ok(())
        }
        [first, rest @ ..] => {
            if table.get(first).is_none() {
                let mut made = Table::new();
                made.set_implicit(true);
                table.insert(first, Item::Table(made));
            }
            let inner = table
                .get_mut(first)
                .and_then(Item::as_table_like_mut)
                .ok_or(())?;
            set(inner, rest, value)
        }
    }
}
