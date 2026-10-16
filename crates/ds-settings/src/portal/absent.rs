//! The portal's stand-in where there is no portal client (not Linux, or no `quire-desktop`): the
//! same two entry points as `portal/bus.rs`, answering the defaults and never changing.

use super::SystemPrefs;
use crate::latest::{self, Receiver};
use ds_core::spawner::Spawner;

/// No portal: the caller falls back to the defaults.
pub(super) async fn read_all() -> Option<SystemPrefs> {
    None
}

/// No portal: the defaults, and nothing ever changes, so the sender drops and the watch stays at
/// its first answer.
pub(super) async fn start(_spawn: &dyn Spawner) -> Receiver<SystemPrefs> {
    latest::channel(SystemPrefs::default()).1
}
