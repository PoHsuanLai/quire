//! A file that is there but cannot be used belongs to the person. It is never taken for a
//! missing one, and never written over: the program runs on defaults in memory, and a save that
//! must happen moves the old file aside first.

use crate::error::SettingsError;
use crate::lenient::{FileText, file_text};
use ds_helpers::{ReadCap, UnixSeconds, move_aside};
use ds_style::space::list::Epoch;
use std::fmt;
use std::path::{Path, PathBuf};

/// A file that exists and could not be read or understood: not text, too big, not a regular
/// file, refused by the system, or not the document it should be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamagedFile {
    /// Which file.
    pub path: PathBuf,
    /// Why it cannot be used.
    pub reason: String,
}

impl fmt::Display for DamagedFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.reason)
    }
}

/// What a save did about a damaged file that was in the way.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct Saved {
    /// Where the damaged file was moved to (`<name>.bad-<seconds>`) before the new one was
    /// written; `None` when the old file was fine or there was none. Worth telling the person.
    pub kept_at: Option<PathBuf>,
}

/// Why the file at `path` cannot be used, `None` when it is missing or `understood` accepts its
/// text.
pub(crate) fn damage(
    path: &Path,
    cap: ReadCap,
    understood: impl FnOnce(&str) -> Result<(), String>,
) -> Option<DamagedFile> {
    let reason = match file_text(path, cap) {
        FileText::Missing => return None,
        FileText::Text(text) => understood(&text).err()?,
        FileText::Unusable(reason) => reason,
    };
    Some(DamagedFile {
        path: path.to_path_buf(),
        reason,
    })
}

/// Before a write at `path`: move the file there aside when it is damaged, so the write does not
/// destroy what the person had.
pub(crate) fn keep_damaged(
    path: &Path,
    cap: ReadCap,
    understood: impl FnOnce(&str) -> Result<(), String>,
) -> Result<Saved, SettingsError> {
    match damage(path, cap, understood) {
        None => Ok(Saved::default()),
        Some(_) => {
            let at = UnixSeconds(u64::try_from(Epoch::now().0).unwrap_or(0));
            Ok(Saved {
                kept_at: Some(move_aside(path, at)?),
            })
        }
    }
}
