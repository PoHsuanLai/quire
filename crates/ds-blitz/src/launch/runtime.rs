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
//! crate can, and `ds-blitz` is quire's one host crate today. So `ds-blitz` owns the runtime:
//! built once, lazily, the first time anything asks to enter it, and kept for the process's
//! life (a `static` is never dropped). Two worker threads, not one: a document's own tasks (a
//! `futures-timer` sleep, a rect probe) and a portal round-trip can both be in flight at once,
//! and nothing quire does needs more than that.
//!
//! `launch` holds the guard for the rest of its call, which blocks until the window closes —
//! i.e., for the process's life. `Harness` holds it as a field, for its own life, so a harness
//! test can construct and drop many harnesses (sequentially, on possibly-different `cargo test`
//! threads) without leaking an ever-growing stack of entries. Guards share one Tokio entry per
//! thread and count themselves, so they may drop in any order (Tokio's own guards panic unless
//! they drop newest first).

use ds::base::spawner::Spawner;
use std::cell::RefCell;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::OnceLock;
use tokio::runtime::{EnterGuard, Handle, Runtime};

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// The process-wide runtime, built on first use.
fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            // A broken invariant in our own code, not malformed input (CONVENTIONS.md §7): the
            // OS refused to start the two worker threads a launched app or a test harness needs.
            .expect("ds-blitz's process-wide tokio runtime could not be started")
    })
}

/// What this thread has of the process-wide runtime: one Tokio entry, held for as long as any
/// [`RuntimeGuard`] lives. Tokio's own `EnterGuard`s panic when they drop out of order, so the
/// guards share this one entry and count themselves instead: whichever drops last lets go, and
/// two guards may drop in any order.
#[derive(Debug, Default)]
struct Entry {
    holders: usize,
    entered: Option<EnterGuard<'static>>,
}

thread_local! {
    static ENTRY: RefCell<Entry> = RefCell::new(Entry::default());
}

/// Enter the process-wide runtime on the calling thread. The caller holds the returned guard
/// for as long as a spawned task must keep working there. Guards may drop in any order; the
/// runtime stays entered until the last one on the thread is gone.
pub fn enter_runtime() -> RuntimeGuard {
    ENTRY.with(|entry| {
        let mut entry = entry.borrow_mut();
        entry.holders += 1;
        if entry.entered.is_none() {
            entry.entered = Some(runtime().enter());
        }
    });
    RuntimeGuard {
        thread: PhantomData,
    }
}

/// A runtime entered on this thread, until it drops (see [`enter_runtime`]).
#[derive(Debug)]
#[must_use = "the runtime is entered only while the guard is alive"]
pub struct RuntimeGuard {
    /// The entry lives in a thread-local, so the guard stays on its thread.
    thread: PhantomData<*const ()>,
}

impl Drop for RuntimeGuard {
    fn drop(&mut self) {
        // `try_with`: a guard dropped during thread teardown finds the entry already gone.
        let released = ENTRY.try_with(|entry| {
            let mut entry = entry.borrow_mut();
            entry.holders = entry.holders.saturating_sub(1);
            (entry.holders == 0).then(|| entry.entered.take())
        });
        // Dropped outside the borrow.
        drop(released);
    }
}

/// How many guards hold the runtime on this thread now: for the order-independence test.
#[cfg(test)]
fn holders() -> usize {
    ENTRY.with(|entry| entry.borrow().holders)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entered() -> bool {
        Handle::try_current().is_ok()
    }

    #[test]
    fn guards_dropped_in_creation_order_do_not_panic_and_the_last_one_leaves() {
        let (first, second) = (enter_runtime(), enter_runtime());
        assert_eq!(holders(), 2);
        drop(first);
        assert!(entered(), "the second guard still holds the runtime");
        drop(second);
        assert!(!entered(), "the last guard leaves the runtime");
        assert_eq!(holders(), 0);
    }

    #[test]
    fn guards_dropped_newest_first_also_work_and_the_runtime_can_be_entered_again() {
        let (first, second) = (enter_runtime(), enter_runtime());
        drop(second);
        assert!(entered());
        drop(first);
        assert!(!entered());
        let again = enter_runtime();
        assert!(entered());
        drop(again);
        assert!(!entered());
    }
}

/// The [`Spawner`] every library below `ds-blitz` is given: it starts tasks on a Tokio runtime.
#[derive(Debug, Clone)]
pub struct TokioSpawner(Handle);

impl TokioSpawner {
    /// The runtime entered on the calling thread: inside `launch`, a `Harness`, or a test's own
    /// `#[tokio::test]`.
    ///
    /// # Panics
    /// When no runtime is entered on this thread, a caller contract like Tokio's own.
    pub fn current() -> Self {
        TokioSpawner(Handle::current())
    }

    /// The runtime `handle` belongs to, for a program that owns its own (a daemon's).
    pub fn on(handle: Handle) -> Self {
        TokioSpawner(handle)
    }
}

impl Spawner for TokioSpawner {
    fn spawn(&self, task: Pin<Box<dyn Future<Output = ()> + Send>>) {
        self.0.spawn(task);
    }
}
