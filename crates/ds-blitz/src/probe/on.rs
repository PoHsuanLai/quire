//! The probe with the feature on: the file a window writes.

use super::snapshot::snapshot;
use crate::node_ref::DocRef;
use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The variable that names the directory the files go to; unset, the probe is off.
const DIR_VAR: &str = "QUIRE_DEBUG_PROBE";

/// Windows the process has given a probe so far.
static WINDOWS: AtomicUsize = AtomicUsize::new(0);

/// One window's probe file and what it holds now.
#[derive(Debug)]
pub(crate) struct Probe {
    file: Option<PathBuf>,
    last: RefCell<String>,
}

impl Default for Probe {
    /// The probe of the next window: on when `QUIRE_DEBUG_PROBE` names a directory.
    fn default() -> Probe {
        let file = std::env::var_os(DIR_VAR).map(|dir| {
            let n = WINDOWS.fetch_add(1, Ordering::Relaxed);
            let _ = std::fs::create_dir_all(&dir);
            PathBuf::from(dir).join(format!("{}-{n}.tsv", std::process::id()))
        });
        Probe {
            file,
            last: RefCell::default(),
        }
    }
}

impl Probe {
    /// Write what `doc` shows now, when it differs from the file. The file is replaced whole
    /// (a rename), so a reader never sees half of one.
    pub(crate) fn publish(&self, doc: &DocRef) {
        let Some(file) = &self.file else { return };
        let Some(text) = doc.read(snapshot) else {
            return;
        };
        if *self.last.borrow() == text {
            return;
        }
        let partial = file.with_extension("tmp");
        if std::fs::write(&partial, &text).is_ok() && std::fs::rename(&partial, file).is_ok() {
            self.last.replace(text);
        }
    }
}
