//! Reading and writing any settings file (design/22-SETTINGS.md section 2): a lenient read in
//! which a bad value costs only its own key, and an atomic temp-and-rename write.
//!
//! A file is named by a typed [`Settings<T>`] constant: `appearance.toml` is
//! [`crate::appearance_file::APPEARANCE`], `spaces.json` is [`crate::spaces::SPACES`], and a
//! consumer declares its own (sill's `settings.toml`) the same way. [`crate::watch`] watches
//! any of them.

use crate::error::SettingsError;
use crate::lenient::{lenient, lenient_json};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// `appearance.toml`'s mailo import, where it has always been reachable
/// (`ds_settings::file::load_or_import`).
pub use crate::appearance_file::load_or_import;

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
    /// `text` as `T`: a key that is not valid costs only itself; text that is not this format
    /// at all is `T::default()`.
    pub fn decode<T>(self, text: &str) -> T
    where
        T: Serialize + DeserializeOwned + Default,
    {
        match self {
            Format::Toml => lenient(text),
            Format::Json => lenient_json(text),
        }
    }

    /// `value` as this format's text.
    pub fn encode<T: Serialize>(self, value: &T) -> Result<String, SettingsError> {
        Ok(match self {
            Format::Toml => toml::to_string(value)?,
            Format::Json => serde_json::to_string_pretty(value)? + "\n",
        })
    }
}

/// One settings file on disk: a directory, a name and a format.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SettingsFile {
    /// The program's config directory (`$XDG_CONFIG_HOME/quire`).
    pub dir: PathBuf,
    /// The file's name inside it.
    pub name: FileName,
    /// How it is written.
    pub format: Format,
}

impl SettingsFile {
    /// The file's full path.
    pub fn path(&self) -> PathBuf {
        self.dir.join(self.name.0)
    }
}

/// A settings file that holds a `T`, not yet placed in a directory: declared once as a
/// constant, then [`Settings::at`] a directory, or loaded, saved and watched straight from it.
pub struct Settings<T> {
    /// The file's name.
    pub name: FileName,
    /// How it is written.
    pub format: Format,
    of: PhantomData<fn() -> T>,
}

impl<T> Settings<T> {
    /// The file `name`, written as `format`, holding a `T`.
    pub const fn new(name: FileName, format: Format) -> Self {
        Settings {
            name,
            format,
            of: PhantomData,
        }
    }

    /// This file inside `dir`.
    pub fn at(&self, dir: &Path) -> SettingsFile {
        SettingsFile {
            dir: dir.to_path_buf(),
            name: self.name,
            format: self.format,
        }
    }
}

impl<T> Settings<T>
where
    T: Serialize + DeserializeOwned + Default,
{
    /// [`load`] this file from `dir`.
    pub fn load(&self, dir: &Path) -> T {
        load(&self.at(dir))
    }

    /// [`save`] `value` as this file in `dir`.
    pub fn save(&self, dir: &Path, value: &T) -> Result<(), SettingsError> {
        save(&self.at(dir), value)
    }
}

// By hand: a derive would ask `T` for the same traits, and a descriptor is copyable whatever
// it describes.
impl<T> Clone for Settings<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Settings<T> {}

impl<T> fmt::Debug for Settings<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Settings")
            .field("name", &self.name)
            .field("format", &self.format)
            .finish()
    }
}

/// The stored value, or the defaults when there is none or it cannot be read.
///
/// Not an error the user needs to see: a missing or damaged file means the program looks as it
/// did on first run, and a bad value costs only its own key.
pub fn load<T>(file: &SettingsFile) -> T
where
    T: Serialize + DeserializeOwned + Default,
{
    match std::fs::read_to_string(file.path()) {
        Ok(text) => file.format.decode(&text),
        Err(_) => T::default(),
    }
}

/// Write `value` as `file`, creating its directory if needed.
///
/// The bytes land in a temporary file beside it and are renamed into place, so a crash
/// mid-write cannot leave a half-written file, and a directory watch sees one rename rather
/// than a truncated file.
pub fn save<T: Serialize>(file: &SettingsFile, value: &T) -> Result<(), SettingsError> {
    let io = |path: &Path| {
        let path = path.to_path_buf();
        move |source| SettingsError::Io { path, source }
    };
    std::fs::create_dir_all(&file.dir).map_err(io(&file.dir))?;
    let path = file.path();
    let tmp = path.with_extension("part");
    let body = file.format.encode(value)?;
    std::fs::write(&tmp, body).map_err(io(&tmp))?;
    std::fs::rename(&tmp, &path).map_err(io(&path))
}
