//! The roster as a hook: [`RosterState`] in a signal, with the settle timers started for it.
//! The timers belong to the hook's owner and drop with it; a timer that finds the roster gone
//! stops (`crate::task`, sill FINDINGS Q45).

use super::presence::Exit;
use super::roster::{RosterEntry, RosterState, RowPitch, StayError, Stayed};
use super::roster_rest::{RestQueue, RestTimer};
use super::settle::settle;
use crate::components::vocab::{Emphasis, StaggerIndex};
use crate::root::env::{Env, use_env_signal};
use crate::task::{Gone, spawn_in, try_get, try_set};
use crate::time::sleep;
use dioxus::core::{Task, current_scope_id};
use dioxus::prelude::*;

/// A live roster: read its entries in render, start exits from handlers.
#[derive(Debug, PartialEq)]
pub struct Roster<K: 'static> {
    pub(super) state: Signal<RosterState<K>>,
    env: Signal<Env>,
    pub(super) scope: ScopeId,
    pub(super) rest: Signal<Option<RestTimer>>,
    pub(super) rest_queue: CopyValue<RestQueue>,
    pub(super) exits: Signal<Vec<ExitTimer<K>>>,
}

/// The settle timer of one leaving row, kept so a stay (or a second leave) can cancel it.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct ExitTimer<K> {
    pub(super) key: K,
    pub(super) task: Task,
}

impl<K: 'static> Clone for Roster<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K: 'static> Copy for Roster<K> {}

impl<K: Clone + PartialEq + 'static> Roster<K> {
    /// The rows to draw, leaving and healing ones included.
    pub fn entries(&self) -> Vec<RosterEntry<K>> {
        self.state.read().entries().to_vec()
    }

    /// Start `key`'s exit; it is dropped, and the rows below heal, when the exit settles. An
    /// unread (`Emphasis::Strong`) row exits at `--t-big-heavy`.
    pub fn leave(&self, key: K, exit: Exit, emphasis: Emphasis) {
        let _ = self.try_leave(key, exit, emphasis);
    }

    fn try_leave(&self, key: K, exit: Exit, emphasis: Emphasis) -> Result<(), Gone> {
        let (next, anim) = try_get(self.state)?.leave(&key, exit, emphasis);
        try_set(self.state, next)?;
        let length = settle(anim, self.level()?, StaggerIndex::default());
        self.cancel_exit(&key)?;
        let roster = *self;
        let settling = key.clone();
        let task = spawn_in(self.scope, async move {
            sleep(length).await;
            let _ = roster.forget_exit(&settling);
            if roster.update(|state| state.settled(&settling)).is_ok() {
                // A task, not a render: it may start the rest timer itself.
                roster.schedule_rest();
            }
        });
        let mut exits = try_get(self.exits)?;
        exits.push(ExitTimer { key, task });
        try_set(self.exits, exits)
    }

    /// Take `key`'s exit back while it plays: the row is present again where it was, its
    /// settle timer is cancelled, and so nothing below it heals (an undo before the row was
    /// dropped). The consumer lists the key again in the same handler, so the next reconcile
    /// keeps the row. A row that is not leaving is unchanged; a key the roster no longer holds
    /// is [`StayError::UnknownKey`] (its exit settled: list it again and it enters).
    pub fn stay(&self, key: K) -> Result<Stayed, StayError> {
        let (next, stayed) = try_get(self.state)
            .map_err(|Gone| StayError::Unmounted)?
            .stay(&key);
        try_set(self.state, next).map_err(|Gone| StayError::Unmounted)?;
        if stayed == Ok(Stayed::Restored) {
            self.cancel_exit(&key)
                .map_err(|Gone| StayError::Unmounted)?;
        }
        stayed
    }

    /// Cancel `key`'s pending exit timer, if it has one.
    pub(super) fn cancel_exit(&self, key: &K) -> Result<(), Gone> {
        if let Some(timer) = self.forget_exit(key)? {
            timer.task.cancel();
        }
        Ok(())
    }

    /// Drop `key`'s exit timer from the list, handing it back.
    pub(super) fn forget_exit(&self, key: &K) -> Result<Option<ExitTimer<K>>, Gone> {
        let mut exits = try_get(self.exits)?;
        let Some(at) = exits.iter().position(|timer| &timer.key == key) else {
            return Ok(None);
        };
        let timer = exits.remove(at);
        try_set(self.exits, exits)?;
        Ok(Some(timer))
    }

    pub(super) fn update(
        &self,
        step: impl FnOnce(RosterState<K>) -> RosterState<K>,
    ) -> Result<(), Gone> {
        let next = step(try_get(self.state)?);
        try_set(self.state, next)
    }

    pub(super) fn level(&self) -> Result<crate::appearance::MotionLevel, Gone> {
        Ok(try_get(self.env)?.resolved.motion)
    }
}

/// A roster over `keys`, which the consumer passes on every render. Stagger is capped at 12.
///
/// The first render shows every key entering; after that, a change in `keys` reconciles
/// (new keys enter, keys removed without an exit drop at once). `pitch` is read on the first
/// render only. Reconciling is pure and happens in the render; the rest timer it needs is
/// started after the render, by an effect ([`Roster::queue_rest`]), never from the body.
pub fn use_roster<K: Clone + PartialEq + 'static>(keys: Vec<K>, pitch: RowPitch) -> Roster<K> {
    let roster = use_roster_parts(&keys, pitch);
    let mut seen = use_hook(|| {
        roster.queue_rest();
        CopyValue::new(keys.clone())
    });
    if *seen.peek() != keys {
        let _ = roster.update(|state| state.reconcile(&keys));
        roster.queue_rest();
        seen.set(keys);
    }
    roster
}

/// The roster's hooks, first showing `keys`: the state and the timers' signals.
pub(super) fn use_roster_parts<K: Clone + PartialEq + 'static>(
    keys: &[K],
    pitch: RowPitch,
) -> Roster<K> {
    Roster {
        state: use_signal(|| RosterState::first_show(keys, pitch)),
        env: use_env_signal(),
        scope: use_hook(current_scope_id),
        rest: use_signal(|| None),
        rest_queue: use_hook(|| CopyValue::new(RestQueue::Idle)),
        exits: use_signal(Vec::new),
    }
}
