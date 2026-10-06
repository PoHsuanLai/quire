//! Is the tool there? A look along `PATH`, nothing run.

use crate::capability::Executable;
use crate::catalog::Entry;
use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// Whether a tool can be run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presence {
    /// An executable for it is on `PATH`.
    Present,
    /// None is.
    Missing,
}

/// The `PATH` and the `os-release` file to read; handed in so a test uses a scratch directory.
#[derive(Debug, Clone)]
pub struct Environment {
    /// A `PATH`-style list of directories.
    pub path: OsString,
    /// The `os-release` file (read, never written).
    pub os_release: std::path::PathBuf,
}

impl Environment {
    /// This process's `PATH` and `/etc/os-release`.
    pub fn system() -> Environment {
        Environment {
            path: std::env::var_os("PATH").unwrap_or_default(),
            os_release: "/etc/os-release".into(),
        }
    }
}

fn is_executable(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn on_path(path: &OsString, executable: &Executable) -> bool {
    std::env::split_paths(path)
        .filter(|dir| dir.is_absolute())
        .any(|dir| is_executable(&dir.join(executable.as_str())))
}

/// Whether any of the entry's probe executables is on `path`.
pub fn presence(path: &OsString, entry: &Entry) -> Presence {
    match entry.probe.iter().any(|exe| on_path(path, exe)) {
        true => Presence::Present,
        false => Presence::Missing,
    }
}

#[cfg(test)]
mod tests {
    use super::{Presence, presence};
    use crate::capability::Capability;
    use crate::catalog::Catalog;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn probing_walks_path_for_an_executable_file() {
        let root = std::env::temp_dir().join(format!("ds-helpers-probe-{}", std::process::id()));
        let (bin, other) = (root.join("bin"), root.join("other"));
        std::fs::create_dir_all(&bin).expect("scratch");
        std::fs::create_dir_all(&other).expect("scratch");
        let make = |dir: &std::path::Path, name: &str, mode: u32| {
            let file = dir.join(name);
            std::fs::write(&file, "#!/bin/sh\n").expect("file");
            std::fs::set_permissions(&file, std::fs::Permissions::from_mode(mode)).expect("mode");
        };
        make(&bin, "heif-dec", 0o755);
        make(&bin, "plain-file", 0o644);
        std::fs::create_dir_all(bin.join("a-directory")).expect("dir");
        let catalog = Catalog::parse(
            "[heic]\ntool=\"x\"\npurpose=\"y\"\nprobe=[\"heif-convert\",\"heif-dec\"]\n\
             [data]\ntool=\"x\"\npurpose=\"y\"\nprobe=[\"plain-file\"]\n\
             [dir]\ntool=\"x\"\npurpose=\"y\"\nprobe=[\"a-directory\"]\n",
        )
        .expect("parses");
        let entry = |name: &str| {
            catalog
                .get(&Capability::new(name).expect("cap"))
                .expect("entry")
        };
        let path = std::env::join_paths([&other, &bin]).expect("path");
        let none = std::env::join_paths([&other]).expect("path");
        let table = [
            ("any one probe is enough", "heic", &path, Presence::Present),
            ("not on this path", "heic", &none, Presence::Missing),
            ("not executable", "data", &path, Presence::Missing),
            ("a directory is not a tool", "dir", &path, Presence::Missing),
        ];
        for (name, cap, path, want) in table {
            assert_eq!(presence(path, entry(cap)), want, "{name}");
        }
        let _ = std::fs::remove_dir_all(&root);
    }
}
