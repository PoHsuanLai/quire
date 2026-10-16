//! Writers of one path in one process take turns.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

/// The lock of every path being written. Process-wide by nature: two callers that never met must
/// still queue for the same file. An entry no writer holds is dropped on the next acquisition.
static HELD: Mutex<BTreeMap<PathBuf, Arc<Mutex<()>>>> = Mutex::new(BTreeMap::new());

/// Run `body` while holding `path`'s lock.
pub(super) fn with<T>(path: &Path, body: impl FnOnce() -> T) -> T {
    let slot = {
        let mut table = HELD.lock().unwrap_or_else(PoisonError::into_inner);
        table.retain(|_, slot| Arc::strong_count(slot) > 1);
        Arc::clone(table.entry(path.to_path_buf()).or_default())
    };
    let _turn = slot.lock().unwrap_or_else(PoisonError::into_inner);
    body()
}
