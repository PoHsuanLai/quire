//! What a settings file is: a serde type that names its file and its format.

use crate::error::SettingsError;
use serde::Serialize;
use serde::de::DeserializeOwned;

/// A settings file's name inside its program's config directory: `appearance.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileName(pub &'static str);

/// How a settings file is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    /// TOML: every hand-edited settings file.
    Toml,
    /// JSON: machine-written stores such as `spaces.json`.
    Json,
}

impl Format {
    /// `value` as this format's text.
    pub fn encode<T: Serialize>(self, value: &T) -> Result<String, SettingsError> {
        Ok(match self {
            Format::Toml => toml::to_string(value)?,
            Format::Json => serde_json::to_string_pretty(value)? + "\n",
        })
    }
}

/// A type stored as one settings file. [`crate::Store`] is the only way to load, save and watch
/// it: a file is read leniently (a bad value costs only its own key), written atomically, and a
/// key the type does not read is reported and never kept.
pub trait SettingsDoc:
    Serialize + DeserializeOwned + Default + Clone + PartialEq + 'static
{
    /// The file's name in the program's config directory.
    const FILE: FileName;
    /// How the file is written.
    const FORMAT: Format;
}
