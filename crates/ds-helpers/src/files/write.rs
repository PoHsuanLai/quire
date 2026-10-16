//! Writing a file so that a reader sees the old bytes or the new ones, never a mixture, and so
//! that the new ones survive a power cut.

use crate::files::{link, lock};
use std::ffi::{OsStr, OsString};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// How many temporary names are tried before the write gives up (each is unique per process, so
/// a clash means leftovers from another one).
const TEMP_TRIES: u32 = 32;

/// The numbers that make the temporary names of one process distinct.
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A write that did not happen.
#[derive(Debug, thiserror::Error)]
#[error("{path}: {source}")]
pub struct WriteError {
    /// The file being written (after any symlink was followed), or the directory that could not
    /// be synced.
    pub path: PathBuf,
    /// Why.
    pub source: io::Error,
}

impl WriteError {
    pub(super) fn at(path: &Path, source: io::Error) -> Self {
        WriteError {
            path: path.to_path_buf(),
            source,
        }
    }
}

/// The permissions a file gets when `write_atomic` creates it. A file that already exists keeps
/// its own, whatever this says.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NewFileMode {
    /// Readable and writable by the person only (0600): for files that carry names, paths,
    /// messages or secrets.
    OwnerOnly,
    /// The system default, the process's umask applied to 0666.
    Umask,
}

/// Write `body` as the file at `path`; the directory must exist.
///
/// The bytes go to a uniquely named temporary file in the same directory (created exclusively),
/// are flushed to disk with `sync_all`, renamed over `path`, and the directory is synced so the
/// rename itself survives a power cut. A crash leaves at worst a stray `.<name>.<pid>.<n>.tmp`
/// beside an intact `path`; a failure removes its own temporary file.
///
/// A symlink at `path` is followed and its target replaced, so a dotfile-managed link stays a
/// link. An existing file's permissions are kept; a new file gets `new_file`. Writers of the same
/// file in this process take turns, so their temporary files and renames never interleave.
pub fn write_atomic(path: &Path, body: &[u8], new_file: NewFileMode) -> Result<(), WriteError> {
    let target = link::resolve(path).map_err(|source| WriteError::at(path, source))?;
    lock::with(&target, || publish(&target, body, new_file))
        .map_err(|source| WriteError::at(&target, source))
}

/// The temporary file, rename and directory sync, for a caller that already holds the path's lock.
pub(super) fn publish(target: &Path, body: &[u8], new_file: NewFileMode) -> io::Result<()> {
    let dir = match target.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    let name = target
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "the path has no file name"))?;
    let kept = std::fs::metadata(target)
        .ok()
        .filter(std::fs::Metadata::is_file)
        .map(|meta| meta.permissions());
    let (temp_path, mut temp) = create_temp(dir, name, new_file)?;
    let written = temp
        .write_all(body)
        .and_then(|()| match kept {
            Some(permissions) => temp.set_permissions(permissions),
            None => Ok(()),
        })
        .and_then(|()| temp.sync_all());
    drop(temp);
    let moved = written.and_then(|()| std::fs::rename(&temp_path, target));
    if moved.is_err() {
        // Best effort: the original is untouched either way, and a leftover is only clutter.
        let _ = std::fs::remove_file(&temp_path);
    }
    moved?;
    sync_dir(dir)
}

fn create_temp(dir: &Path, name: &OsStr, new_file: NewFileMode) -> io::Result<(PathBuf, File)> {
    for _ in 0..TEMP_TRIES {
        let number = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut temp_name = OsString::from(".");
        temp_name.push(name);
        temp_name.push(format!(".{}.{number}.tmp", std::process::id()));
        let temp_path = dir.join(temp_name);
        match open_new(&temp_path, new_file) {
            Ok(file) => return Ok((temp_path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "no free temporary file name",
    ))
}

#[cfg_attr(not(unix), allow(unused_variables))]
fn open_new(path: &Path, new_file: NewFileMode) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    if new_file == NewFileMode::OwnerOnly {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

/// Flush the directory entry of a rename to disk.
fn sync_dir(dir: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        File::open(dir)?.sync_all()
    }
    #[cfg(not(unix))]
    {
        let _ = dir;
        Ok(())
    }
}
