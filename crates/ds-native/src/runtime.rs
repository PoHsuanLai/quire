//! A process-wide Tokio runtime, entered by whichever caller needs `tokio::spawn` to work.
//!
//! `ds_settings::use_environment` calls `tokio::spawn` directly in two places that are not
//! optional for a Blitz consumer: the desktop-portal watch (`ds-settings/src/portal.rs`, over
//! zbus's `tokio` feature) and the file-watch debounce (`ds_settings::watch`, `tokio::time::
//! timeout` inside a spawned task). Both panic ("there is no reactor running") unless a runtime
//! is entered on the calling thread first (CONSUMING.md section 3, the gap this module closes).
//!
//! Neither `ds` nor `ds-settings` may depend on a renderer or a windowing stack
//! (`scripts/check-boundary.sh` forbids `tokio` itself to `ds`, and every render/window crate
//! to both), so neither of them can own a *host thread* to enter a runtime on — only a host
//! crate can, and `ds-native` is quire's one host crate today. So `ds-native` owns the runtime:
//! built once, lazily, the first time anything asks to enter it, and kept for the process's
//! life (a `static` is never dropped). Two worker threads, not one: a document's own tasks (a
//! `futures-timer` sleep, a rect probe) and a portal round-trip can both be in flight at once,
//! and nothing quire does needs more than that.
//!
//! `launch` holds the guard for the rest of its call, which blocks until the window closes —
//! i.e., for the process's life. `Harness` holds it as a field, for its own life, so a harness
//! test can construct and drop many harnesses (sequentially, on possibly-different `cargo test`
//! threads) without leaking an ever-growing stack of entries: each `Harness` enters once and
//! exits when it is dropped.

use std::sync::OnceLock;
use tokio::runtime::{EnterGuard, Runtime};

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// The process-wide runtime, built on first use.
fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            // A broken invariant in our own code, not malformed input (CONVENTIONS.md #5): the
            // OS refused to start the two worker threads a launched app or a test harness needs.
            .expect("ds-native's process-wide tokio runtime could not be started")
    })
}

/// Enter the process-wide runtime on the calling thread. The caller holds the returned guard
/// for as long as a spawned task must keep working there.
pub(crate) fn enter() -> EnterGuard<'static> {
    runtime().enter()
}
