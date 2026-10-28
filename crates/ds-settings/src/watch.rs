//! Live reload: a `notify` watch on the config directory, not the file, because editors and the
//! atomic writer both replace the inode by rename (design/22-SETTINGS.md section 2). A burst of
//! events settles for [`DEBOUNCE`], then the whole file is re-read.

use crate::doc::{FileName, SettingsDoc};
use crate::error::SettingsError;
use crate::latest::{self, Receiver, Sender};
use crate::lenient::{FileText, Loaded, Read, SETTINGS_CAP, file_text, read_within};
use crate::store::Store;
use ds_core::spawner::Spawner;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::time::Duration;

/// How long the watch waits for a burst of events to settle before re-reading.
pub const DEBOUNCE: Duration = Duration::from_millis(30);

/// How long the watch waits before reading again a file the system would not give it (too many
/// open files, permissions mid-change). No file event follows such a failure, so the watch asks
/// again itself; the wait doubles each time.
const RETRY_FIRST: Duration = Duration::from_millis(250);

/// How many times a refused read is asked again before the watch waits for the next file event.
const RETRIES: u32 = 6;

/// Whether a [`Watch`] is looking at the disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchState {
    /// Every settled change arrives through [`Watch::changed`].
    Live,
    /// The directory could not be watched: the value stays as first read and `changed` ends.
    Blind {
        /// Why the watch could not start.
        reason: String,
    },
}

/// A running watch on one settings file. Dropping it stops the watch.
#[derive(Debug)]
pub struct Watch<D> {
    changes: Receiver<Loaded<D>>,
    state: WatchState,
    _watcher: Option<RecommendedWatcher>,
}

impl<D: SettingsDoc> Watch<D> {
    /// Wait for the next settled change: the whole re-read file, never a partial one, with what
    /// was refused or dropped in it. A file that is not valid text keeps the last good value and
    /// reports itself invalid. `None` once the watch has stopped.
    pub async fn changed(&mut self) -> Option<Loaded<D>> {
        self.changes.changed().await
    }

    /// The file as last read.
    pub fn current(&self) -> D {
        self.changes.latest().value
    }

    /// Whether the directory is being watched.
    pub fn state(&self) -> &WatchState {
        &self.state
    }
}

impl<D: SettingsDoc + Send> Watch<D> {
    pub(crate) fn start(store: &Store, spawn: &dyn Spawner) -> Watch<D> {
        // The watcher starts before the first read, so a write that lands in between is seen as
        // an event and re-read, not missed until the next edit.
        let armed = arm::<D>(store);
        let initial = store.load::<D>();
        let (out, changes) = latest::channel(initial.clone());
        match armed {
            Ok((watcher, signals)) => {
                let path = store.path::<D>().unwrap_or_default();
                spawn.spawn(Box::pin(settle(path, initial.value, signals, out)));
                Watch {
                    changes,
                    state: WatchState::Live,
                    _watcher: Some(watcher),
                }
            }
            Err(error) => Watch {
                changes,
                state: WatchState::Blind {
                    reason: error.to_string(),
                },
                _watcher: None,
            },
        }
    }
}

/// Whether `event` is a write to the file `name` itself: not the temp file the atomic writer
/// (or an editor) writes and renames away, not an unrelated file in the same directory (another
/// watched settings file included), and not an `Access` event: opening the file to read it
/// (which every re-read this watch does, and every editor's own load, triggers) is not a change
/// and must not re-arm the debounce, or a watcher re-reading the file becomes a change that
/// makes it re-read the file forever.
fn touches_the_file(event: &notify::Event, name: FileName) -> bool {
    !event.kind.is_access()
        && event
            .paths
            .iter()
            .any(|path| path.file_name() == Some(OsStr::new(name.0)))
}

/// Start the directory watch; its events arrive on the returned receiver. The watcher is the
/// caller's to keep alive.
fn arm<D: SettingsDoc + Send>(
    store: &Store,
) -> Result<(RecommendedWatcher, Receiver<()>), SettingsError> {
    let dir = store.require_dir()?;
    std::fs::create_dir_all(&dir).map_err(|source| SettingsError::Io {
        path: dir.clone(),
        source,
    })?;
    let (signal, signals) = latest::channel(());
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res
            && touches_the_file(&event, D::FILE)
        {
            // The receiver is gone once the watch task ended; there is nobody left to signal.
            let _ = signal.send(());
        }
    })?;
    watcher.watch(&dir, RecursiveMode::NonRecursive)?;
    Ok((watcher, signals))
}

/// Re-read `path` after every settled burst of events and publish it, until the watcher or the
/// receiver is dropped. `good` is the last value that read cleanly.
// The debounce is a wall-clock wait on the spawner's own thread: `ds_core::time::clock::sleep` follows the clock
// installed on its caller's thread and is not `Send`, and a task on a runtime has neither.
async fn settle<D: SettingsDoc + Send>(
    path: PathBuf,
    mut good: D,
    mut signals: Receiver<()>,
    out: Sender<Loaded<D>>,
) {
    while signals.changed().await.is_some() {
        // Coalesce the rest of the burst: a rename can fire more than one raw event, and this
        // crate's own atomic writer plus a watching editor can both touch the file in quick
        // succession. Every further event restarts the window.
        loop {
            futures_timer::Delay::new(DEBOUNCE).await;
            if !signals.has_changed() {
                break;
            }
            signals.catch_up();
        }
        let (loaded, mut refused) = reread(&path, &mut good);
        if out.send(loaded).is_err() {
            return;
        }
        // A refused read raises no file event, so ask again with a growing wait, unless the
        // file changes meanwhile (then the outer loop reads it). Only the answer that ends the
        // refusal is published; the same refusal again tells nobody anything new.
        let mut wait = RETRY_FIRST;
        for _ in 0..RETRIES {
            if !refused {
                break;
            }
            futures_timer::Delay::new(wait).await;
            if signals.has_changed() {
                break;
            }
            wait = wait.saturating_mul(2);
            let (again, next) = reread(&path, &mut good);
            refused = next;
            if !refused && out.send(again).is_err() {
                return;
            }
        }
    }
}

/// The file at `path` as the watch publishes it, updating `good` when it read cleanly or is gone,
/// and whether the system refused it (nothing will announce when it stops, so read it again later;
/// a file that was read, is gone or is wrongly written needs no retry, as an edit announces its fix).
fn reread<D: SettingsDoc + Send>(path: &std::path::Path, good: &mut D) -> (Loaded<D>, bool) {
    match file_text(path, SETTINGS_CAP) {
        FileText::Text(text) => match read_within::<D>(&text, D::FORMAT, &D::limits()) {
            Read::Garbled { reason } => (Loaded::garbled(good.clone(), reason), false),
            Read::Loaded(loaded) => {
                *good = loaded.value.clone();
                (loaded, false)
            }
        },
        FileText::Unusable(reason) => (Loaded::garbled(good.clone(), reason), true),
        FileText::Missing => {
            *good = D::default();
            (Loaded::default(), false)
        }
    }
}
