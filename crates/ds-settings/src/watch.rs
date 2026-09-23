//! Live reload: a `notify` watch on the config directory, not the file, because editors and the
//! atomic writer both replace the inode by rename (design/22-SETTINGS.md section 2).

use crate::error::SettingsError;
use crate::file::{self, FILE_NAME};
use crate::settings::AppearanceFile;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// How long the watch waits for a burst of events to settle before re-reading.
pub const DEBOUNCE: Duration = Duration::from_millis(30);

/// A running watch on one directory's `appearance.toml`.
#[derive(Debug)]
pub struct AppearanceWatch {
    changes: tokio::sync::watch::Receiver<AppearanceFile>,
    _watcher: RecommendedWatcher,
}

impl AppearanceWatch {
    /// Wait for the next settled change and return the whole re-read file, never a partial one.
    /// `None` once the watch has stopped.
    pub async fn changed(&mut self) -> Option<AppearanceFile> {
        if self.changes.changed().await.is_err() {
            return None;
        }
        Some(self.changes.borrow_and_update().clone())
    }

    /// The file as last read.
    pub fn current(&self) -> AppearanceFile {
        self.changes.borrow().clone()
    }
}

/// Whether `event` is a write to `appearance.toml` itself: not the temp file the atomic writer
/// (or an editor) writes and renames away, not an unrelated file in the same directory, and not
/// an `Access` event — opening the file to read it (which every re-read this watch does, and
/// every editor's own load, triggers) is not a change and must not re-arm the debounce, or a
/// watcher re-reading the file becomes a change that makes it re-read the file forever.
fn touches_the_file(event: &notify::Event) -> bool {
    !event.kind.is_access()
        && event
            .paths
            .iter()
            .any(|path| path.file_name() == Some(OsStr::new(FILE_NAME)))
}

/// Start watching `dir` for changes to `appearance.toml`.
pub fn watch(dir: &Path) -> Result<AppearanceWatch, SettingsError> {
    let io = |path: &Path| {
        let path = path.to_path_buf();
        move |source| SettingsError::Io { path, source }
    };
    std::fs::create_dir_all(dir).map_err(io(dir))?;

    let initial = file::load(dir);
    let (settled_tx, mut settled_rx) = tokio::sync::mpsc::unbounded_channel::<()>();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res
            && touches_the_file(&event)
        {
            // The receiver may already be gone if the watch task ended; a dropped signal is
            // fine, there is nobody left to re-read for.
            let _ = settled_tx.send(());
        }
    })?;
    watcher.watch(dir, RecursiveMode::NonRecursive)?;

    let (changes_tx, changes_rx) = tokio::sync::watch::channel(initial);
    let dir_owned: PathBuf = dir.to_path_buf();
    tokio::spawn(async move {
        loop {
            if settled_rx.recv().await.is_none() {
                return;
            }
            // Coalesce the rest of the burst: a rename can fire more than one raw event, and
            // this crate's own atomic writer plus a watching editor can both touch the file in
            // quick succession. Reset the debounce window on every further signal.
            loop {
                match tokio::time::timeout(DEBOUNCE, settled_rx.recv()).await {
                    Ok(Some(())) => continue,
                    Ok(None) => return,
                    Err(_timed_out) => break,
                }
            }
            if changes_tx.send(file::load(&dir_owned)).is_err() {
                return;
            }
        }
    });

    Ok(AppearanceWatch {
        changes: changes_rx,
        _watcher: watcher,
    })
}

#[cfg(test)]
mod tests {
    use super::{DEBOUNCE, watch};
    use crate::settings::AppearanceFile;
    use ds::Theme;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    /// A fresh directory under the system temp dir, removed on drop. (No `tempfile`: it is not
    /// in the pinned dependency block — same helper as `file.rs`'s tests.)
    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            static NEXT: AtomicU32 = AtomicU32::new(0);
            let n = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("ds-settings-watch-test-{}-{n}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            TempDir(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// Generous slack over the 30 ms debounce for a test runner under load, without being so
    /// long the test hangs if the watch is broken.
    const MARGIN: Duration = Duration::from_millis(2_000);

    #[tokio::test]
    async fn a_renamed_write_fires_exactly_one_change() {
        let dir = TempDir::new();
        let initial = AppearanceFile::default();
        crate::file::save(dir.path(), &initial).unwrap_or_else(|e| panic!("{e}"));

        let mut appearance_watch = watch(dir.path()).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(appearance_watch.current(), initial);

        let mut changed = AppearanceFile::default();
        changed.appearance.theme = Theme::Dark;
        // The same atomic temp-and-rename write the real writer does, not an in-place write.
        crate::file::save(dir.path(), &changed).unwrap_or_else(|e| panic!("{e}"));

        let got = tokio::time::timeout(MARGIN, appearance_watch.changed())
            .await
            .unwrap_or_else(|_| panic!("no change observed within the debounce plus margin"))
            .unwrap_or_else(|| panic!("the watch stopped"));
        assert_eq!(got, changed);
        assert_eq!(appearance_watch.current(), changed);

        // Exactly one: nothing further arrives once the burst from the rename has settled.
        let extra = tokio::time::timeout(DEBOUNCE * 4, appearance_watch.changed()).await;
        assert!(extra.is_err(), "expected no further change, got {extra:?}");
    }

    #[tokio::test]
    async fn a_watch_started_before_the_file_exists_sees_its_first_write() {
        let dir = TempDir::new();
        let mut appearance_watch = watch(dir.path()).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(appearance_watch.current(), AppearanceFile::default());

        let mut written = AppearanceFile::default();
        written.appearance.theme = Theme::Light;
        crate::file::save(dir.path(), &written).unwrap_or_else(|e| panic!("{e}"));

        let got = tokio::time::timeout(MARGIN, appearance_watch.changed())
            .await
            .unwrap_or_else(|_| panic!("no change observed within the debounce plus margin"))
            .unwrap_or_else(|| panic!("the watch stopped"));
        assert_eq!(got, written);
    }
}
