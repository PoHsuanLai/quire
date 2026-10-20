//! The folders desktop entries are searched in.

use crate::catalog::data_dirs;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The folder of desktop entries inside a data directory.
const APPLICATIONS: &str = "applications";
/// The system Flatpak install's exports.
const FLATPAK_SYSTEM: &str = "/var/lib/flatpak/exports/share/applications";
/// Snap's exports.
const SNAP: &str = "/var/lib/snapd/desktop/applications";

/// The three environment values that decide where entries are, as the caller read them. Keeping
/// them in a value makes the resolution pure: a test hands in what it wants.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct XdgEnv {
    /// `XDG_DATA_HOME`.
    pub data_home: Option<OsString>,
    /// `XDG_DATA_DIRS`.
    pub data_dirs: Option<OsString>,
    /// `HOME`, for the default of `XDG_DATA_HOME`.
    pub home: Option<PathBuf>,
}

impl XdgEnv {
    /// The values of this process's environment.
    pub fn from_process() -> XdgEnv {
        XdgEnv {
            data_home: std::env::var_os("XDG_DATA_HOME"),
            data_dirs: std::env::var_os("XDG_DATA_DIRS"),
            home: std::env::var_os("HOME").map(PathBuf::from),
        }
    }

    /// The person's data folder: `XDG_DATA_HOME` when it is an absolute path, else
    /// `$HOME/.local/share`; none when neither is usable.
    pub fn data_home(&self) -> Option<PathBuf> {
        let set = self
            .data_home
            .as_ref()
            .map(PathBuf::from)
            .filter(|dir| dir.is_absolute());
        set.or_else(|| {
            let home = self.home.as_ref().filter(|home| home.is_absolute())?;
            Some(home.join(".local/share"))
        })
    }

    /// The system data folders: `XDG_DATA_DIRS` without its empty and relative entries, or
    /// `/usr/local/share:/usr/share` when it is unset or empty.
    pub fn data_dirs(&self) -> Vec<PathBuf> {
        data_dirs(self.data_dirs.as_deref())
    }

    /// The `applications` folders, most important first: the person's, each system one, then the
    /// app stores' export folders (the user's Flatpak install, the system's, Snap's) that are not
    /// already listed. A session that has sourced the stores' profile scripts lists them itself,
    /// at the position it chose, and keeps it.
    pub fn applications_dirs(&self) -> Vec<PathBuf> {
        let home = self.data_home();
        let mut dirs: Vec<PathBuf> = home
            .iter()
            .map(PathBuf::as_path)
            .chain(self.data_dirs().iter().map(PathBuf::as_path))
            .map(|dir| dir.join(APPLICATIONS))
            .collect();
        for store in store_dirs(home.as_deref()) {
            if !dirs.contains(&store) {
                dirs.push(store);
            }
        }
        dirs
    }
}

/// The app stores' export folders, the person's own first.
fn store_dirs(data_home: Option<&Path>) -> Vec<PathBuf> {
    data_home
        .map(|home| home.join("flatpak/exports/share").join(APPLICATIONS))
        .into_iter()
        .chain([PathBuf::from(FLATPAK_SYSTEM), PathBuf::from(SNAP)])
        .collect()
}
