//! What every test shares: a scratch directory and Extension folders written into it.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

/// One test's own directory under the system temp dir, removed on drop. (No `tempfile`: it is not
/// in the pinned dependency block.)
pub struct Scratch(PathBuf);

/// A look entry, as a manifest writes it.
pub const LOOK: &str =
    "[[provides]]\ncapability = \"look\"\nstylesheet = \"look.css\"\nstyles_for = 3\n";

/// A file-icons entry.
pub const ICONS: &str = "[[provides]]\ncapability = \"file_icons\"\ndir = \"file-icons\"\n";

impl Scratch {
    pub fn new() -> Scratch {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("ds-extensions-test-{}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        Scratch(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    /// A data root named `name` (it stands in for one `$XDG_DATA_HOME` or `$XDG_DATA_DIRS` entry).
    pub fn root(&self, name: &str) -> PathBuf {
        let root = self.0.join(name);
        fs::create_dir_all(&root).unwrap_or_else(|e| panic!("{}: {e}", root.display()));
        root
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A manifest with the envelope filled in and `entries` after it.
pub fn manifest(id: &str, name: &str, version: &str, entries: &str) -> String {
    format!(
        "id = \"{id}\"\nname = \"{name}\"\nprotocol = 1\nversion = \"{version}\"\n\
         licence = \"MIT\"\nsummary = \"A test Extension\"\n\n{entries}"
    )
}

/// The folder `<root>/quire/extensions/<id>` with `text` as its manifest; returns the folder.
pub fn install(root: &Path, id: &str, text: &str) -> PathBuf {
    let folder = root.join("quire").join("extensions").join(id);
    fs::create_dir_all(&folder).unwrap_or_else(|e| panic!("{}: {e}", folder.display()));
    write(&folder.join("extension.toml"), text);
    folder
}

/// An Extension of the given `id` (named after it, version 1.0.0) with `entries` in its manifest.
pub fn install_entries(root: &Path, id: &str, entries: &str) -> PathBuf {
    install(root, id, &manifest(id, id, "1.0.0", entries))
}

/// An Extension with a look: its manifest and a small stylesheet.
pub fn install_look(root: &Path, id: &str, name: &str, version: &str) -> PathBuf {
    let folder = install(root, id, &manifest(id, name, version, LOOK));
    write(&folder.join("look.css"), ".ds { --accent: red; }\n");
    folder
}

/// `text` as the file at `path`, its folder made.
pub fn write(path: &Path, text: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|e| panic!("{}: {e}", parent.display()));
    }
    fs::write(path, text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
}
