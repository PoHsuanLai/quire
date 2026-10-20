//! Finding the `.desktop` files under the `applications` folders.

use super::desktop_id::DesktopId;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// One desktop file found and the id its place gives it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryFile {
    /// Where the file is.
    pub path: PathBuf,
    /// Its id below the folder it was found in.
    pub id: DesktopId,
}

/// Every `.desktop` file under `roots`, in order: a folder's own files by name, then its
/// subfolders by name, each root before the next. A root or folder that cannot be read is
/// skipped. A folder is entered once however many links lead to it, so a link back up the tree
/// ends the walk. The same id can come twice (one per root); [`first_of_each_id`] drops the
/// shadowed ones.
pub fn walk_applications(roots: &[PathBuf]) -> Vec<EntryFile> {
    let mut walk = Walk::default();
    for root in roots {
        walk.enter(root, Path::new(""));
    }
    walk.found
}

/// `files` without any file whose id an earlier one already has: the entry in the more important
/// folder shadows the others, as the specification says.
pub fn first_of_each_id(files: Vec<EntryFile>) -> Vec<EntryFile> {
    let mut seen = BTreeSet::new();
    files
        .into_iter()
        .filter(|file| seen.insert(file.id.clone()))
        .collect()
}

/// The walk so far: the folders entered (by real path) and the files met.
#[derive(Default)]
struct Walk {
    entered: BTreeSet<PathBuf>,
    found: Vec<EntryFile>,
}

impl Walk {
    /// Walks `dir`, whose place below its root is `relative`.
    fn enter(&mut self, dir: &Path, relative: &Path) {
        let real = dir.canonicalize().unwrap_or_else(|_| dir.to_owned());
        if !self.entered.insert(real.clone()) {
            return;
        }
        let Ok(read) = std::fs::read_dir(&real) else {
            return;
        };
        let mut names: Vec<_> = read.filter_map(Result::ok).map(|e| e.file_name()).collect();
        names.sort_unstable();
        let (folders, files): (Vec<_>, Vec<_>) =
            names.into_iter().partition(|name| real.join(name).is_dir());
        for name in files {
            let below = relative.join(&name);
            let path = dir.join(&name);
            if let Some(id) = DesktopId::from_relative(&below).filter(|_| path.is_file()) {
                self.found.push(EntryFile { path, id });
            }
        }
        for name in folders {
            self.enter(&dir.join(&name), &relative.join(&name));
        }
    }
}
