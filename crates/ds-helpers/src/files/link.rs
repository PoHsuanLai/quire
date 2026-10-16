//! Following a symlink to the file a write is really for.

use std::io;
use std::path::{Path, PathBuf};

/// The most links followed before the path is called a loop.
const MAX_LINKS: usize = 40;

/// `path` with every symlink at its end followed, so a write replaces the file the link points at
/// and the link stays a link. A link whose target is missing resolves to that target path, so the
/// write creates it.
pub(super) fn resolve(path: &Path) -> io::Result<PathBuf> {
    let mut current = path.to_path_buf();
    for _ in 0..MAX_LINKS {
        match std::fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => {
                let target = std::fs::read_link(&current)?;
                current = match current.parent() {
                    Some(parent) => parent.join(target),
                    None => target,
                };
            }
            _ => return Ok(current),
        }
    }
    Err(io::Error::other("too many levels of symbolic links"))
}
