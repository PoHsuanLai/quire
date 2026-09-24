//! Tasks the design system spawns, and how they end (sill FINDINGS Q45).
//!
//! Every task quire spawns is owned by a scope, and dioxus drops it when that scope drops. A
//! task spawned for a hook's owner rather than for the component whose handler started it
//! ([`spawn_in`]) is registered with the owner the same way, through the owner's own `spawn`:
//! `Runtime::spawn(scope, …)` alone runs the task *for* the scope without registering it, so it
//! outlived the scope and its signals, and the first `set` after the wait panicked with
//! "ValueDroppedError" (a palette unmounted mid entrance took the shell down).
//!
//! A task's body still writes only through [`try_set`] and reads only through [`try_get`]: a
//! signal can belong to another scope than the task (a hub's timer writing a child's state),
//! and a write to one that is gone ends the task's work instead of panicking.

use dioxus::core::{Runtime, ScopeId, Task};
use dioxus::prelude::*;
use std::future::Future;

/// A signal whose owner has dropped: the task that wanted it has nothing left to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Gone;

/// Run `future` as a task of `scope`: dioxus drops it, mid-wait or not, when `scope` drops.
/// `scope` must still be mounted; callers read a signal it owns with [`try_get`] first.
pub(crate) fn spawn_in(scope: ScopeId, future: impl Future<Output = ()> + 'static) -> Task {
    Runtime::current().in_scope(scope, || spawn(future))
}

/// Set `signal` to `value`, or report that its owner is gone.
pub(crate) fn try_set<T: 'static>(signal: Signal<T>, value: T) -> Result<(), Gone> {
    let mut signal = signal;
    signal
        .try_write()
        .map(|mut slot| *slot = value)
        .map_err(|_| Gone)
}

/// Set `signal` to `value` when it differs, or report that its owner is gone.
pub(crate) fn try_set_if_changed<T: PartialEq + 'static>(
    signal: Signal<T>,
    value: T,
) -> Result<(), Gone> {
    if *signal.try_peek().map_err(|_| Gone)? == value {
        return Ok(());
    }
    try_set(signal, value)
}

/// A copy of `signal`'s value, or [`Gone`].
pub(crate) fn try_get<T: Clone + 'static>(signal: Signal<T>) -> Result<T, Gone> {
    signal
        .try_peek()
        .map(|value| value.clone())
        .map_err(|_| Gone)
}
