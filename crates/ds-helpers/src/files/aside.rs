//! Keeping a damaged file instead of writing over it.

use crate::files::{lock, write::WriteError};
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

/// A moment as seconds since the Unix epoch, for a `.bad` name. The caller reads the clock, so a
/// test names the instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnixSeconds(pub u64);

/// Rename the file at `path` to `<name>.bad-<seconds>` beside it (with `-1`, `-2`, ... when that
/// name is taken) and return the new path. Call this before writing over a file that exists but
/// could not be read or understood, so the person's bytes are never lost to a default.
///
/// A symlink is moved as the link it is. The rename takes the same per-path turn as
/// [`crate::write_atomic`].
pub fn move_aside(path: &Path, at: UnixSeconds) -> Result<PathBuf, WriteError> {
    lock::with(path, || rename_aside(path, at)).map_err(|source| WriteError::at(path, source))
}

fn rename_aside(path: &Path, at: UnixSeconds) -> io::Result<PathBuf> {
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "the path has no file name"))?;
    let free = (0u32..)
        .map(|n| {
            let mut aside = OsString::from(name);
            aside.push(format!(".bad-{}", at.0));
            if n > 0 {
                aside.push(format!("-{n}"));
            }
            path.with_file_name(aside)
        })
        .find(|candidate| std::fs::symlink_metadata(candidate).is_err())
        .ok_or_else(|| io::Error::other("no free name"))?;
    std::fs::rename(path, &free)?;
    if let Some(dir) = path.parent().filter(|dir| !dir.as_os_str().is_empty()) {
        #[cfg(unix)]
        std::fs::File::open(dir)?.sync_all()?;
        #[cfg(not(unix))]
        let _ = dir;
    }
    Ok(free)
}
