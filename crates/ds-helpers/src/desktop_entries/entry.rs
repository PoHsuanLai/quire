//! Desktop entries read from disk, and the one a window belongs to.

use super::desktop_id::DesktopId;
use super::identity::AppIdentity;
use super::match_app::{AppKeys, match_app};
use super::walk::{first_of_each_id, walk_applications};
use crate::{DESKTOP_ENTRY_CAP, DesktopFile, read_text_capped};
use std::path::PathBuf;

/// A parsed desktop entry with the id its place gave it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationEntry {
    /// Where the file is.
    pub path: PathBuf,
    /// Its desktop id.
    pub id: DesktopId,
    /// What window matching reads from it.
    pub keys: AppKeys,
    /// The whole file, for the keys a caller reads itself (`Name`, `Exec`, ...).
    pub file: DesktopFile,
}

impl ApplicationEntry {
    /// `Icon=`: a theme name or an absolute path.
    pub fn icon(&self) -> Option<String> {
        self.value("Icon")
    }

    /// The identity the entry declares ([`AppIdentity`]); none when it has no `[Desktop Entry]`.
    pub fn identity(&self) -> Option<AppIdentity> {
        let group = self.file.entry()?;
        Some(AppIdentity::from_entry(self.id.clone(), group))
    }

    fn value(&self, key: &str) -> Option<String> {
        value_of(&self.file, key)
    }
}

/// The trimmed, non-empty default value of `key` in the `[Desktop Entry]` group.
fn value_of(file: &DesktopFile, key: &str) -> Option<String> {
    let value = file.entry()?.string(key)?.trim().to_owned();
    (!value.is_empty()).then_some(value)
}

/// Every entry under `dirs` (the `applications` folders, most important first), each id once,
/// the most important folder's file standing for it. A file that cannot be read, is over
/// [`DESKTOP_ENTRY_CAP`], is not text, or has no `[Desktop Entry]` group is skipped.
pub fn load_entries(dirs: &[PathBuf]) -> Vec<ApplicationEntry> {
    first_of_each_id(walk_applications(dirs))
        .into_iter()
        .filter_map(|found| {
            let text = read_text_capped(&found.path, DESKTOP_ENTRY_CAP).ok()?;
            let file = DesktopFile::parse(&text, DESKTOP_ENTRY_CAP).ok()?;
            file.entry()?;
            let keys = AppKeys {
                id: found.id.stem().to_owned(),
                startup_wm_class: value_of(&file, "StartupWMClass"),
                flatpak: value_of(&file, "X-Flatpak"),
            };
            Some(ApplicationEntry {
                path: found.path,
                id: found.id,
                keys,
                file,
            })
        })
        .collect()
}

/// The entry of `entries` that a window with `app_id` (and X11 `wm_class`) belongs to, by
/// [`match_app`].
pub fn entry_for_window<'a>(
    entries: &'a [ApplicationEntry],
    app_id: &str,
    wm_class: Option<&str>,
) -> Option<&'a ApplicationEntry> {
    let keys: Vec<&AppKeys> = entries.iter().map(|entry| &entry.keys).collect();
    entries.get(match_app(app_id, wm_class, &keys)?)
}
