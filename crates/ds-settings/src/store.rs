//! Reading and writing settings files (design/22-SETTINGS.md section 2): a lenient read in
//! which a bad value costs only its own key, and an atomic temp-and-rename write.
//!
//! A [`Store`] is one program's config directory. Every file in it is a [`SettingsDoc`]; there
//! is no other load or save path.

use crate::doc::SettingsDoc;
use crate::error::SettingsError;
use crate::lenient::{FileText, Loaded, Read, file_text, read};
use crate::root::{AppName, ConfigRoot};
use crate::watch::Watch;
use ds_core::spawner::Spawner;
use std::path::{Path, PathBuf};

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

    /// `D` as stored, or its defaults when there is no file or it cannot be read: a missing or
    /// damaged file means the program looks as it did on first run, and a bad value costs only
    /// its own key. What was refused or dropped is in the result, for the caller to report.
    pub fn load<D: SettingsDoc>(&self) -> Loaded<D> {
        let Some(path) = self.path::<D>() else {
            return Loaded::default();
        };
        match file_text(&path) {
            FileText::Text(text) => Self::decode(&text),
            FileText::Missing => Loaded::default(),
            FileText::NotText(reason) => Loaded::garbled(D::default(), reason),
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
    /// The bytes land in a temporary file beside it and are renamed into place, so a crash
    /// mid-write cannot leave a half-written file, and a directory watch sees one rename rather
    /// than a truncated one.
    pub fn save<D: SettingsDoc>(&self, doc: &D) -> Result<(), SettingsError> {
        let dir = self.require_dir()?;
        let io = |path: &Path| {
            let path = path.to_path_buf();
            move |source| SettingsError::Io { path, source }
        };
        std::fs::create_dir_all(&dir).map_err(io(&dir))?;
        let path = dir.join(D::FILE.0);
        write_atomic(&path, D::FORMAT.encode(doc)?.as_bytes())
    }

    /// Watch `D`'s file: every settled change is re-read whole. The watch runs as a task on
    /// `spawn`.
    pub fn watch<D: SettingsDoc + Send>(&self, spawn: &dyn Spawner) -> Watch<D> {
        Watch::start(self, spawn)
    }
}

/// Write `body` to `path` through a temporary file beside it and a rename, creating nothing
/// else: the directory must exist.
pub(crate) fn write_atomic(path: &Path, body: &[u8]) -> Result<(), SettingsError> {
    let io = |path: &Path| {
        let path = path.to_path_buf();
        move |source| SettingsError::Io { path, source }
    };
    let tmp = path.with_extension("part");
    std::fs::write(&tmp, body).map_err(io(&tmp))?;
    std::fs::rename(&tmp, path).map_err(io(path))
}
