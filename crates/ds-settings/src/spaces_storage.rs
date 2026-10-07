//! An app's Spaces on disk: `spaces.json` in its config directory and `today.json` in its state
//! directory (design/21-SPACES.md section 13).
//!
//! These are the app's data, not settings: the payload is the app's own type, so the lenient
//! key-by-key settings reader does not apply; `ds_style::space::list` reads the file leniently
//! itself. A missing or corrupt file is `None` / an empty Today, never a crash, and every write
//! is atomic (a temporary file and a rename), as `appearance.toml`'s is.

use crate::error::SettingsError;
use crate::lenient::{FileText, file_text};
use crate::root::{AppName, ConfigRoot};
use crate::store::write_atomic;
use ds_style::space::list::{Epoch, Spaces, Today};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

const SPACES_FILE: &str = "spaces.json";
const TODAY_FILE: &str = "today.json";

/// Whether a load step changed the Spaces and so they need writing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fixup {
    /// Nothing changed.
    Kept,
    /// The Spaces changed.
    Changed,
}

/// Where the Spaces were found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// Read from `spaces.json`.
    Stored,
    /// There was no usable file: the first run built them (and they were written).
    FirstRun,
}

/// The app's directories for its Spaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpacesStorage {
    config: Option<PathBuf>,
    state: Option<PathBuf>,
}

impl SpacesStorage {
    /// `app`'s directories inside `root`.
    pub fn new(root: &ConfigRoot, app: AppName) -> Self {
        SpacesStorage {
            config: root.dir(app),
            state: root.state_dir(app),
        }
    }

    /// Explicit directories, for a program that keeps its files elsewhere. `None` keeps the
    /// Spaces for the session only.
    pub fn at(config: Option<PathBuf>, state: Option<PathBuf>) -> Self {
        SpacesStorage { config, state }
    }

    /// The Spaces as stored, or `None` when there is no file, it is not JSON, or it holds no
    /// Space. `raw_fix` sees the parsed JSON first: a one-time import of what an older version
    /// wrote (mailo gives a Space with no theme the window-wide one).
    pub fn load_spaces<P: DeserializeOwned, R: DeserializeOwned>(
        &self,
        raw_fix: impl FnOnce(&mut serde_json::Value),
    ) -> Option<Spaces<P, R>> {
        let FileText::Text(text) = file_text(&self.config.as_ref()?.join(SPACES_FILE)) else {
            return None;
        };
        let mut value = serde_json::from_str(&text).ok()?;
        raw_fix(&mut value);
        serde_json::from_value(value).ok()
    }

    /// The window's first Spaces: the stored ones, else `first_run()`; then `on_load` (a payload
    /// fix-up, such as filling an account's colour). They are written when this was a first
    /// run or `on_load` changed them. Nothing here can fail the caller: a write that cannot
    /// happen leaves the Spaces for the session.
    pub fn boot_spaces<P, R>(
        &self,
        raw_fix: impl FnOnce(&mut serde_json::Value),
        first_run: impl FnOnce() -> Spaces<P, R>,
        on_load: impl FnOnce(&mut Spaces<P, R>) -> Fixup,
    ) -> (Spaces<P, R>, Origin)
    where
        P: Serialize + DeserializeOwned,
        R: Serialize + DeserializeOwned,
    {
        let (mut spaces, origin) = match self.load_spaces(raw_fix) {
            Some(spaces) => (spaces, Origin::Stored),
            None => (first_run(), Origin::FirstRun),
        };
        let fixup = on_load(&mut spaces);
        if origin == Origin::FirstRun || fixup == Fixup::Changed {
            let _ = self.save_spaces(&spaces);
        }
        (spaces, origin)
    }

    /// Write `spaces` to `spaces.json`, creating the directory if needed.
    pub fn save_spaces<P: Serialize, R: Serialize>(
        &self,
        spaces: &Spaces<P, R>,
    ) -> Result<(), SettingsError> {
        write_json(self.config.as_deref(), SPACES_FILE, spaces)
    }

    /// Today as stored, or an empty one when there is no file or it cannot be read.
    pub fn load_today<I: DeserializeOwned, K: DeserializeOwned>(&self) -> Today<I, K> {
        let Some(FileText::Text(text)) = self
            .state
            .as_ref()
            .map(|dir| file_text(&dir.join(TODAY_FILE)))
        else {
            return Today::default();
        };
        serde_json::from_str(&text).unwrap_or_default()
    }

    /// Today as it should start: stored, with the entries idle past their time dropped, and
    /// written back when any were.
    pub fn boot_today<I, K>(&self, now: Epoch) -> Today<I, K>
    where
        I: Serialize + DeserializeOwned,
        K: Serialize + DeserializeOwned,
    {
        let mut today = self.load_today::<I, K>();
        if today.prune(now) > 0 {
            let _ = self.save_today(&today);
        }
        today
    }

    /// Write `today` to `today.json`, creating the directory if needed.
    pub fn save_today<I: Serialize, K: Serialize>(
        &self,
        today: &Today<I, K>,
    ) -> Result<(), SettingsError> {
        write_json(self.state.as_deref(), TODAY_FILE, today)
    }
}

fn write_json(dir: Option<&Path>, name: &str, value: &impl Serialize) -> Result<(), SettingsError> {
    let dir = dir.ok_or(SettingsError::NoConfigDir {
        app: "this program",
    })?;
    std::fs::create_dir_all(dir).map_err(|source| SettingsError::Io {
        path: dir.to_path_buf(),
        source,
    })?;
    let body = serde_json::to_string_pretty(value)? + "\n";
    write_atomic(&dir.join(name), body.as_bytes())
}
