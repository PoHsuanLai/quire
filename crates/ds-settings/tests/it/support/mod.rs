//! What every settings integration test shares: a scratch directory standing in for
//! `$XDG_CONFIG_HOME`. No test reads or writes the real `~/.config`.

use ds_settings::{AppName, ConfigRoot, Store};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

/// The program name every test's store uses.
pub const APP: AppName = AppName("probe");

/// One test's own directory under the system temp dir, removed on drop. (No `tempfile`: it is not
/// in the pinned dependency block.)
pub struct Scratch(PathBuf);

impl Scratch {
    pub fn new() -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("ds-settings-test-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        Scratch(path)
    }

    /// The store whose program directory is inside this scratch directory.
    pub fn store(&self) -> Store {
        Store::new(ConfigRoot::Scratch(self.0.clone()), APP)
    }

    /// The program directory itself, where the store's files land.
    pub fn app_dir(&self) -> PathBuf {
        self.0.join(APP.0)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
