//! Live reload: a `notify` watch on the config directory, not the file, because editors and the
//! atomic writer both replace the inode by rename (design/22-SETTINGS.md section 2).
#![allow(unused_variables, dead_code)] // Freeze stubs: remove with the last todo!().

use crate::error::SettingsError;
use crate::settings::AppearanceFile;
use std::path::Path;
use std::time::Duration;

/// How long the watch waits for a burst of events to settle before re-reading.
pub const DEBOUNCE: Duration = Duration::from_millis(30);

/// A running watch on one directory's `appearance.toml`.
#[derive(Debug)]
pub struct AppearanceWatch {
    changes: tokio::sync::watch::Receiver<AppearanceFile>,
    _watcher: notify::RecommendedWatcher,
}

impl AppearanceWatch {
    /// Wait for the next settled change and return the whole re-read file, never a partial one.
    /// `None` once the watch has stopped.
    pub async fn changed(&mut self) -> Option<AppearanceFile> {
        todo!()
    }

    /// The file as last read.
    pub fn current(&self) -> AppearanceFile {
        todo!()
    }
}

/// Start watching `dir` for changes to `appearance.toml`.
pub fn watch(dir: &Path) -> Result<AppearanceWatch, SettingsError> {
    todo!()
}
