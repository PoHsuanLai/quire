//! An app's Spaces on disk: `spaces.json` in its config directory and `today.json` in its state
//! directory (design/21-SPACES.md section 13).
//!
//! These are the app's data, not settings: the payload is the app's own type, so the lenient
//! key-by-key settings reader does not apply; `ds_style::space::list` reads the file leniently
//! itself. A missing file is a first run (or an empty Today). A file that is there and cannot be
//! used is never taken for a missing one: the app runs on defaults in memory, the file is left
//! as it was, and the first save moves it aside to `<name>.bad-<seconds>`. Reads are capped and
//! never block on a FIFO, and every write is durable and atomic (`ds_helpers::write_atomic`).

use crate::damaged::{DamagedFile, Saved, keep_damaged};
use crate::error::SettingsError;
use crate::lenient::{FileText, file_text};
use crate::root::{AppName, ConfigRoot};
use ds_helpers::{NewFileMode, ReadCap, write_atomic};
use ds_style::space::list::{Epoch, Spaces, Today};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

const SPACES_FILE: &str = "spaces.json";
const TODAY_FILE: &str = "today.json";

/// The most either file may hold. Names and a payload per Space are kilobytes; 16 MiB is already
/// a mistake, and a bigger file would stall the app's start.
const STORAGE_CAP: ReadCap = ReadCap::mib(16);

/// Whether a load step changed the Spaces and so they need writing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fixup {
    /// Nothing changed.
    Kept,
    /// The Spaces changed.
    Changed,
}

/// Where the Spaces were found.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Origin {
    /// Read from `spaces.json`.
    Stored,
    /// There was no file: the first run built them (and they were written).
    FirstRun,
    /// `spaces.json` is there and could not be read or understood. The Spaces are the first run's,
    /// for this run only, and the file is left exactly as it was; the first save moves it to
    /// `spaces.json.bad-<seconds>` before writing. Tell the person.
    Damaged(DamagedFile),
}

/// What reading `spaces.json` found.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum SpacesLoad<P, R> {
    /// The stored Spaces.
    Stored(Spaces<P, R>),
    /// No file (or no directory to keep one in): a first run.
    Missing,
    /// A file that cannot be used. It is not a first run, and must not be written over.
    Damaged(DamagedFile),
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

    /// The Spaces as stored: [`SpacesLoad::Missing`] only when there is no file, and
    /// [`SpacesLoad::Damaged`] when there is one that is unreadable, too big, not JSON, or holds no
    /// Space. `raw_fix` sees the parsed JSON first: a one-time import of what an older version
    /// wrote (mailo gives a Space with no theme the window-wide one).
    pub fn load_spaces<P: DeserializeOwned, R: DeserializeOwned>(
        &self,
        raw_fix: impl FnOnce(&mut serde_json::Value),
    ) -> SpacesLoad<P, R> {
        let Some(dir) = self.config.as_ref() else {
            return SpacesLoad::Missing;
        };
        let path = dir.join(SPACES_FILE);
        let damaged = |reason: String| {
            SpacesLoad::Damaged(DamagedFile {
                path: path.clone(),
                reason,
            })
        };
        match file_text(&path, STORAGE_CAP) {
            FileText::Missing => SpacesLoad::Missing,
            FileText::Unusable(reason) => damaged(reason),
            FileText::Text(text) => {
                let mut value = match serde_json::from_str(&text) {
                    Ok(value) => value,
                    Err(error) => return damaged(format!("not valid JSON: {error}")),
                };
                raw_fix(&mut value);
                match serde_json::from_value(value) {
                    Ok(spaces) => SpacesLoad::Stored(spaces),
                    Err(error) => damaged(error.to_string()),
                }
            }
        }
    }

    /// The window's first Spaces: the stored ones, else `first_run()`; then `on_load` (a payload
    /// fix-up, such as filling an account's colour). They are written when this was a first
    /// run or `on_load` changed them. Nothing here can fail the caller: a write that cannot
    /// happen leaves the Spaces for the session and is reported on stderr.
    ///
    /// A `spaces.json` that is there and cannot be used is [`Origin::Damaged`]: the Spaces run
    /// from `first_run()` in memory and nothing is written, so the person's file survives until
    /// they choose to save (see [`SpacesStorage::save_spaces`]).
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
            SpacesLoad::Stored(spaces) => (spaces, Origin::Stored),
            SpacesLoad::Missing => (first_run(), Origin::FirstRun),
            SpacesLoad::Damaged(damaged) => (first_run(), Origin::Damaged(damaged)),
        };
        let fixup = on_load(&mut spaces);
        let write = match origin {
            Origin::FirstRun => true,
            Origin::Damaged(_) => false,
            Origin::Stored => fixup == Fixup::Changed,
        };
        if write && let Err(error) = self.save_spaces(&spaces) {
            eprintln!("quire settings: spaces.json was not saved: {error}");
        }
        (spaces, origin)
    }

    /// Write `spaces` to `spaces.json` (readable by the person only), creating the directory if
    /// needed. A `spaces.json` already there that cannot be read as Spaces is first moved aside
    /// to `spaces.json.bad-<seconds>`: [`Saved::kept_at`] says where. (A file in an older layout
    /// that `boot_spaces` imported through its `raw_fix` is kept the same way, as a backup.)
    pub fn save_spaces<P: Serialize + DeserializeOwned, R: Serialize + DeserializeOwned>(
        &self,
        spaces: &Spaces<P, R>,
    ) -> Result<Saved, SettingsError> {
        write_json::<Spaces<P, R>>(self.config.as_deref(), SPACES_FILE, spaces)
    }

    /// Today as stored, or an empty one when there is no file or it cannot be used. A file that
    /// cannot be used is left on disk as it was; the first save moves it aside (see
    /// [`SpacesStorage::save_today`]).
    pub fn load_today<I: DeserializeOwned, K: DeserializeOwned>(&self) -> Today<I, K> {
        let Some(FileText::Text(text)) = self
            .state
            .as_ref()
            .map(|dir| file_text(&dir.join(TODAY_FILE), STORAGE_CAP))
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
        if today.prune(now) > 0
            && let Err(error) = self.save_today(&today)
        {
            eprintln!("quire settings: today.json was not saved: {error}");
        }
        today
    }

    /// Write `today` to `today.json` (readable by the person only), creating the directory if
    /// needed. A `today.json` already there that cannot be read as Today is first moved aside
    /// to `today.json.bad-<seconds>`.
    pub fn save_today<I: Serialize + DeserializeOwned, K: Serialize + DeserializeOwned>(
        &self,
        today: &Today<I, K>,
    ) -> Result<Saved, SettingsError> {
        write_json::<Today<I, K>>(self.state.as_deref(), TODAY_FILE, today)
    }
}

/// `value` as `name` in `dir`, moving a file there that is not a `T` aside first.
fn write_json<T: Serialize + DeserializeOwned>(
    dir: Option<&Path>,
    name: &str,
    value: &T,
) -> Result<Saved, SettingsError> {
    let dir = dir.ok_or(SettingsError::NoConfigDir {
        app: "this program",
    })?;
    std::fs::create_dir_all(dir).map_err(|source| SettingsError::Io {
        path: dir.to_path_buf(),
        source,
    })?;
    let path = dir.join(name);
    let body = serde_json::to_string_pretty(value)? + "\n";
    let saved = keep_damaged(&path, STORAGE_CAP, |text| {
        serde_json::from_str::<T>(text)
            .map(|_| ())
            .map_err(|error| error.to_string())
    })?;
    write_atomic(&path, body.as_bytes(), NewFileMode::OwnerOnly)?;
    Ok(saved)
}
