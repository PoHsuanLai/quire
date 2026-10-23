//! The `file_icons` capability: a folder of file-type pictures (design: Extensions, sections 4.2
//! and 6).
//!
//! ```toml
//! [[provides]]
//! capability = "file_icons"
//! dir = "file-icons"            # relative to the Extension's folder
//! ```
//!
//! This crate parses and checks the entry only. Reading the pictures and resolving a type to one
//! is the file-icon resolver's (a later phase).

use std::fs;
use std::path::{Path, PathBuf};

use bayonet::manifest::{Entry, Refusal};
use serde::Deserialize;

use crate::fault::Fault;

/// What a manifest's `file_icons` entry says, as written.
#[derive(Debug, Deserialize)]
struct Wire {
    dir: PathBuf,
}

/// A `file_icons` entry: the folder of pictures, inside the Extension's folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileIcons {
    dir: PathBuf,
    folder: PathBuf,
}

impl FileIcons {
    /// Reads the entry. The folder path is made absolute inside the Extension's folder
    /// (`Entry::resolve`): an absolute path or a `..` is refused.
    pub(crate) fn from_entry(entry: Entry) -> Result<FileIcons, Refusal<Fault>> {
        let wire: Wire = entry.clone().read::<Wire, Fault>()?;
        let dir = entry.resolve::<Fault>(wire.dir)?;
        let folder = entry.resolve::<Fault>(PathBuf::from("."))?;
        Ok(FileIcons { dir, folder })
    }

    /// The folder of pictures.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// The first problem with the folder this entry names, if any: it is not a folder, or a link
    /// leads out of the Extension's folder.
    pub(crate) fn fault(&self) -> Option<Fault> {
        let is_folder = fs::metadata(&self.dir).is_ok_and(|meta| meta.is_dir());
        if !is_folder {
            return Some(Fault::NotAFolder {
                path: self.dir.clone(),
            });
        }
        match (fs::canonicalize(&self.folder), fs::canonicalize(&self.dir)) {
            (Ok(home), Ok(real)) if !real.starts_with(&home) => {
                Some(Fault::LeadsOut { path: real })
            }
            _ => None,
        }
    }
}
