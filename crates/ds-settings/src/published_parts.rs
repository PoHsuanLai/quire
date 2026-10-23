//! An app's published style parts as data: what a person's stylesheet may rely on in that app
//! (`ds::selectors::AppSelectors`, CONVENTIONS section 16), installed beside its settings schema
//! as `$XDG_DATA_DIRS/quire/selectors/<app>.toml`, so Settings can check a stylesheet against
//! every app's parts without linking any app (customization design section 7.3).
//!
//! The file is two keys: `app = "sill"` and `classes = ["sill-bar", "sill-bar-item", ...]`. A
//! class outside the app's own `<app>-` prefix is dropped on both write and read, so a file cannot
//! claim a `ds-` name or another app's.

use std::io;
use std::path::{Path, PathBuf};

use ds_helpers::{NewFileMode, ReadCap, read_text_capped, write_atomic};
use serde::{Deserialize, Serialize};

use crate::xdg;

/// The most a parts file may hold.
const PARTS_CAP: ReadCap = ReadCap::kib(256);

/// One app's public style parts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishedParts {
    /// The app's short name, lower case: `sill`.
    pub app: String,
    /// Every public class, without the leading dot, each starting with `<app>-`.
    pub classes: Vec<String>,
}

impl PublishedParts {
    /// `app`'s parts: the classes of its `AppSelectors::classes()`, keeping only its own prefix.
    pub fn new(app: &str, classes: Vec<String>) -> PublishedParts {
        PublishedParts {
            app: app.to_owned(),
            classes: own(app, classes),
        }
    }

    /// The TOML text this writes as `<app>.toml`.
    pub fn to_toml(&self) -> String {
        toml::to_string(self).expect("two strings and a list of strings are TOML")
    }

    /// The parts `text` holds, with any class outside the app's prefix dropped, or why it does
    /// not parse.
    pub fn from_toml(text: &str) -> Result<PublishedParts, String> {
        let raw: PublishedParts = toml::from_str(text).map_err(|error| error.to_string())?;
        Ok(PublishedParts::new(&raw.app, raw.classes))
    }

    /// Writes `<app>.toml` into `dir` (creating it), atomically: an app's `--write-selectors`.
    pub fn write_to(&self, dir: &Path) -> io::Result<PathBuf> {
        std::fs::create_dir_all(dir)?;
        let target = dir.join(format!("{}.toml", self.app));
        write_atomic(&target, self.to_toml().as_bytes(), NewFileMode::Umask)
            .map_err(|error| error.source)?;
        Ok(target)
    }
}

/// `classes` that start with `<app>-`.
fn own(app: &str, classes: Vec<String>) -> Vec<String> {
    let prefix = format!("{app}-");
    classes
        .into_iter()
        .filter(|class| class.starts_with(&prefix))
        .collect()
}

/// Every installed app's parts in `dirs` (see [`parts_dirs_from`]). The first file for an app
/// wins, as for settings schemas, so a person's own directory (last) can add an app but not
/// replace an installed one's parts. A file not named for its app, unreadable or malformed is
/// skipped with a line on standard error.
pub fn discover_parts(dirs: &[PathBuf]) -> Vec<PublishedParts> {
    let mut found: Vec<PublishedParts> = Vec::new();
    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        let mut paths: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "toml"))
            .collect();
        paths.sort();
        for path in paths {
            match admit(&path, &found) {
                Ok(parts) => found.push(parts),
                Err(why) => eprintln!("quire selectors: skipping {}: {why}", path.display()),
            }
        }
    }
    found
}

/// The parts file at `path`, if it may join `found`.
fn admit(path: &Path, found: &[PublishedParts]) -> Result<PublishedParts, String> {
    let text = read_text_capped(path, PARTS_CAP).map_err(|error| error.to_string())?;
    let parts = PublishedParts::from_toml(&text)?;
    let named_for = path.file_stem().and_then(|stem| stem.to_str());
    if named_for != Some(parts.app.as_str()) {
        return Err(format!("the file is not named for its app, {}", parts.app));
    }
    if found.iter().any(|earlier| earlier.app == parts.app) {
        return Err(format!(
            "{} already has parts in an earlier directory",
            parts.app
        ));
    }
    Ok(parts)
}

/// The directories [`discover_parts`] scans: each `$XDG_DATA_DIRS` entry's `quire/selectors`,
/// then `$XDG_DATA_HOME/quire/selectors` last, with the environment handed in.
pub fn parts_dirs_from(
    xdg_data_home: Option<std::ffi::OsString>,
    xdg_data_dirs: Option<std::ffi::OsString>,
    home: Option<std::ffi::OsString>,
) -> Vec<PathBuf> {
    xdg::search_dirs(xdg_data_dirs.as_deref())
        .into_iter()
        .chain(xdg::base_dir(
            xdg_data_home.as_deref(),
            home.as_deref(),
            xdg::DATA_HOME_UNDER_HOME,
        ))
        .map(|base| base.join("quire").join("selectors"))
        .collect()
}

/// [`parts_dirs_from`] read from the environment.
pub fn parts_dirs() -> Vec<PathBuf> {
    parts_dirs_from(
        std::env::var_os("XDG_DATA_HOME"),
        std::env::var_os("XDG_DATA_DIRS"),
        std::env::var_os("HOME"),
    )
}
