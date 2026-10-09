//! The Tokio runtime a host enters, so `tokio::spawn` works for what a quire app runs.
//!
//! `ds_settings::use_environment` calls `tokio::spawn` directly in two places that are not
//! optional for a Blitz consumer: the desktop-portal watch (`ds-settings/src/portal.rs`, over
//! zbus's `tokio` feature) and the file-watch debounce (`ds_settings::watch`, `tokio::time::
//! timeout` inside a spawned task). Both panic ("there is no reactor running") unless a runtime
//! is entered on the calling thread first (CONSUMING.md section 3).
//!
//! There is no process-wide runtime. [`launch`](crate::launch) builds one it owns for the call
//! (two worker threads: a document's own tasks and a portal round-trip can both be in flight) or
//! enters the [`Handle`] the app gave with
//! [`AppConfig::with_runtime`](crate::AppConfig::with_runtime). A test driver (`Harness`) and
//! [`MenuExport`](crate::menus) use [`enter_runtime`], which enters a runtime owned by the calling
//! thread. Its guards share one Tokio entry per thread and count themselves, so they may drop in
//! any order (Tokio's own guards panic unless they drop newest first).

use crate::error::RuntimeError;
use ds::base::spawner::Spawner;
use std::cell::RefCell;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use tokio::runtime::{EnterGuard, Handle, Runtime};

/// A runtime of two worker threads, owned by whoever holds it.
pub(super) fn build() -> Result<Runtime, RuntimeError> {
    Ok(tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?)
}

/// What this thread has of its runtime: one Tokio entry, held for as long as any
/// [`RuntimeGuard`] lives. Tokio's own `EnterGuard`s panic when they drop out of order, so the
/// guards share this one entry and count themselves instead: whichever drops last lets go, and
/// two guards may drop in any order.
#[derive(Default)]
struct Entry {
    holders: usize,
    /// Built on the thread's first guard, dropped with the thread.
    runtime: Option<(Runtime, &'static Handle)>,
    entered: Option<EnterGuard<'static>>,
}

thread_local! {
    static ENTRY: RefCell<Entry> = RefCell::new(Entry::default());
}

/// Enter a runtime owned by the calling thread (built on the first call on the thread). The
/// caller holds the returned guard for as long as a spawned task must keep working there. Guards
/// may drop in any order; the runtime stays entered until the last one on the thread is gone.
pub fn enter_runtime() -> Result<RuntimeGuard, RuntimeError> {
    ENTRY.with(|entry| {
        let mut entry = entry.borrow_mut();
        if entry.runtime.is_none() {
            let runtime = build()?;
            // One small `Handle` per thread that ever enters, leaked so the entry can borrow it
            // for `'static` without `unsafe`; the runtime itself is dropped with the thread.
            let handle: &'static Handle = Box::leak(Box::new(runtime.handle().clone()));
            entry.runtime = Some((runtime, handle));
        }
        entry.holders += 1;
        if entry.entered.is_none() {
            let handle = entry.runtime.as_ref().map(|(_, handle)| *handle);
            entry.entered = handle.map(Handle::enter);
        }
        Ok(RuntimeGuard {
            thread: PhantomData,
        })
    })
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
        let (first, second) = (
            enter_runtime().expect("a runtime"),
            enter_runtime().expect("a runtime"),
        );
        assert_eq!(holders(), 2);
        drop(first);
        assert!(entered(), "the second guard still holds the runtime");
        drop(second);
        assert!(!entered(), "the last guard leaves the runtime");
        assert_eq!(holders(), 0);
    }

    #[test]
    fn guards_dropped_newest_first_also_work_and_the_runtime_can_be_entered_again() {
        let (first, second) = (
            enter_runtime().expect("a runtime"),
            enter_runtime().expect("a runtime"),
        );
        drop(second);
        assert!(entered());
        drop(first);
        assert!(!entered());
        let again = enter_runtime().expect("a runtime");
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
