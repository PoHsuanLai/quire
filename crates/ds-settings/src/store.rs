//! Reading and writing settings files (design/22-SETTINGS.md section 2): a lenient read in
//! which a bad value costs only its own key, and an atomic temp-and-rename write.
//!
//! A [`Store`] is one program's config directory. Every file in it is a [`SettingsDoc`]; there
//! is no other load or save path.

use crate::damaged::{InTheWay, Saved, keep_damaged};
use crate::doc::{Format, SettingsDoc};
use crate::error::SettingsError;
use crate::lenient::{FileText, Loaded, Read, SETTINGS_CAP, file_text, read};
use crate::preserve;
use crate::root::{AppName, ConfigRoot};
use crate::watch::Watch;
use ds_core::spawner::Spawner;
use ds_helpers::{NewFileMode, UpdateError, update_atomic, write_atomic};
use std::path::PathBuf;

/// One program's settings directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Store {
    root: ConfigRoot,
    app: AppName,
}

impl Store {
    /// The settings directory of `app` inside `root`.
    pub fn new(root: ConfigRoot, app: AppName) -> Store {
        Store { root, app }
    }

    /// The program's config directory, `None` when the root has no directory for it.
    pub fn dir(&self) -> Option<PathBuf> {
        self.root.dir(self.app)
    }

    /// The program's config directory, or the error a write or a watch reports without one.
    pub(crate) fn require_dir(&self) -> Result<PathBuf, SettingsError> {
        self.dir()
            .ok_or(SettingsError::NoConfigDir { app: self.app.0 })
    }

    /// The path `D` is stored at.
    pub fn path<D: SettingsDoc>(&self) -> Option<PathBuf> {
        self.dir().map(|dir| dir.join(D::FILE.0))
    }

    /// `D` as stored, or its defaults when there is no file or it cannot be used: a missing file
    /// means the program looks as it did on first run, and a bad value costs only its own key.
    /// A file that is there and cannot be used (not text, too big, not a regular file, refused
    /// by the system) is left alone on disk: the defaults are for this run only, and the file is
    /// reported in [`Loaded::invalid`] as wholly invalid. What was refused or dropped is in the
    /// result, for the caller to report.
    pub fn load<D: SettingsDoc>(&self) -> Loaded<D> {
        let Some(path) = self.path::<D>() else {
            return Loaded::default();
        };
        match file_text(&path, SETTINGS_CAP) {
            FileText::Text(text) => Self::decode(&text),
            FileText::Missing => Loaded::default(),
            FileText::Unusable(reason) => Loaded::garbled(D::default(), reason),
        }
    }

    /// `text` read as a `D`; text that is not the document's format at all is the defaults with
    /// the whole file reported invalid.
    pub(crate) fn decode<D: SettingsDoc>(text: &str) -> Loaded<D> {
        match read::<D>(text, D::FORMAT) {
            Read::Loaded(loaded) => loaded,
            Read::Garbled { reason } => Loaded::garbled(D::default(), reason),
        }
    }

    /// Write `doc` as its file, creating the directory if needed.
    ///
    /// A TOML file already there is not replaced but changed: `doc`'s values are laid over the
    /// parsed file, so a key `D` does not read (a newer rev's, a hand-added one), every comment and
    /// the person's formatting stay as they were, and a changed value keeps its spacing and its
    /// trailing comment. A key `D` owns and no longer writes (a `None`, a removed map entry) is
    /// taken out. Other formats, and a file that is not there, get `doc` as encoded.
    ///
    /// The read, the merge and the write happen under the file's lock ([`ds_helpers::update_atomic`]):
    /// writers in this process and in other programs take turns, and the bytes land in a uniquely
    /// named temporary file beside it, are flushed and renamed into place, so a crash mid-write
    /// cannot leave a half-written file and a directory watch sees one rename. A symlinked
    /// settings file is written through.
    ///
    /// A file already there that cannot be used (see [`Store::load`]) or is not this document at
    /// all is first moved aside to `<name>.bad-<seconds>`; [`Saved::kept_at`] says where.
    pub fn save<D: SettingsDoc>(&self, doc: &D) -> Result<Saved, SettingsError> {
        let dir = self.require_dir()?;
        std::fs::create_dir_all(&dir).map_err(|source| SettingsError::Io {
            path: dir.clone(),
            source,
        })?;
        let path = dir.join(D::FILE.0);
        let body = D::FORMAT.encode(doc)?;
        let update = update_atomic(&path, SETTINGS_CAP, NewFileMode::Umask, |existing| {
            merged::<D>(existing, &body)
        });
        match update {
            Ok(()) => Ok(Saved::default()),
            Err(UpdateError::Write(error)) => Err(error.into()),
            // A file that could not be read, or whose text is not this document, is in the way.
            Err(_) => {
                let saved = keep_damaged(&path, SETTINGS_CAP, understood::<D>)?;
                write_atomic(&path, body.as_bytes(), NewFileMode::Umask)?;
                Ok(saved)
            }
        }
    }

    /// Watch `D`'s file: every settled change is re-read whole. The watch runs as a task on
    /// `spawn`.
    pub fn watch<D: SettingsDoc + Send>(&self, spawn: &dyn Spawner) -> Watch<D> {
        Watch::start(self, spawn)
    }
}

/// `Ok` when `text` is a `D`'s file (a bad value costs its own key, so it still is), else why not.
fn understood<D: SettingsDoc>(text: &str) -> Result<(), String> {
    match read::<D>(text, D::FORMAT) {
        Read::Garbled { reason } => Err(reason),
        Read::Loaded(_) => Ok(()),
    }
}

/// The text a save writes: `body` laid over the `existing` TOML file, or `body` itself when there
/// is no file or the format has no comments to keep. [`InTheWay`] when the file is not a `D`.
fn merged<D: SettingsDoc>(existing: Option<String>, body: &str) -> Result<String, InTheWay> {
    let Some(text) = existing else {
        return Ok(body.to_owned());
    };
    match read::<D>(&text, D::FORMAT) {
        Read::Garbled { .. } => Err(InTheWay),
        Read::Loaded(loaded) => match D::FORMAT {
            Format::Toml => preserve::merge(&text, body, &loaded.unknown),
            _ => Ok(body.to_owned()),
        },
    }
}
