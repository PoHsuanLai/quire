//! The roster as a hook: [`RosterState`] in a signal, with the settle timers started for it.
//! The timers belong to the hook's owner and drop with it; a timer that finds the roster gone
//! stops (`crate::task`, sill FINDINGS Q45).

use super::presence::Exit;
use super::roster::{RosterEntry, RosterState, RowPitch, StayError, Stayed};
use super::settle::settle;
use crate::components::vocab::{Emphasis, StaggerIndex};
use crate::root::env::{Env, use_env_signal};
use crate::task::{Gone, spawn_in, try_get, try_set};
use crate::time::sleep;
use dioxus::core::{Task, current_scope_id};
use dioxus::prelude::*;
use std::time::Instant;

/// A live roster: read its entries in render, start exits from handlers.
#[derive(Debug, PartialEq)]
pub struct Roster<K: 'static> {
    state: Signal<RosterState<K>>,
    env: Signal<Env>,
    scope: ScopeId,
    rest_at: Signal<Option<Instant>>,
    exits: Signal<Vec<ExitTimer<K>>>,
}

/// The settle timer of one leaving row, kept so a stay (or a second leave) can cancel it.
#[derive(Debug, Clone, PartialEq)]
struct ExitTimer<K> {
    key: K,
    task: Task,
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
    fn cancel_exit(&self, key: &K) -> Result<(), Gone> {
        if let Some(timer) = self.forget_exit(key)? {
            timer.task.cancel();
        }
        Ok(())
    }

    /// Drop `key`'s exit timer from the list, handing it back.
    fn forget_exit(&self, key: &K) -> Result<Option<ExitTimer<K>>, Gone> {
        let mut exits = try_get(self.exits)?;
        let Some(at) = exits.iter().position(|timer| &timer.key == key) else {
            return Ok(None);
        };
        let timer = exits.remove(at);
        try_set(self.exits, exits)?;
        Ok(Some(timer))
    }

    fn update(&self, step: impl FnOnce(RosterState<K>) -> RosterState<K>) -> Result<(), Gone> {
        let next = step(try_get(self.state)?);
        try_set(self.state, next)
    }

    fn level(&self) -> Result<crate::appearance::MotionLevel, Gone> {
        Ok(try_get(self.env)?.resolved.motion)
    }

    /// Mark the entering and healing rows present once the longest of their animations has
    /// settled. A later call that ends later supersedes an earlier one.
    fn schedule_rest(&self) {
        let _ = self.try_schedule_rest();
    }

    fn try_schedule_rest(&self) -> Result<(), Gone> {
        let level = self.level()?;
        let Some(length) = try_get(self.state)?
            .running()
            .into_iter()
            .map(|(anim, index)| settle(anim, level, index))
            .max()
        else {
            return Ok(());
        };
        let due = Instant::now() + length;
        let due = try_get(self.rest_at)?.map_or(due, |pending| pending.max(due));
        try_set(self.rest_at, Some(due))?;
        let (roster, rest_at) = (*self, self.rest_at);
        spawn_in(self.scope, async move {
            sleep(due.saturating_duration_since(Instant::now())).await;
            if try_get(rest_at) == Ok(Some(due)) && try_set(rest_at, None).is_ok() {
                let _ = roster.update(RosterState::rest);
            }
        });
        Ok(())
    }
}

/// A roster over `keys`, which the consumer passes on every render. Stagger is capped at 12.
///
/// The first render shows every key entering; after that, a change in `keys` reconciles
/// (new keys enter, keys removed without an exit drop at once). `pitch` is read on the first
/// render only.
pub fn use_roster<K: Clone + PartialEq + 'static>(keys: Vec<K>, pitch: RowPitch) -> Roster<K> {
    let env = use_env_signal();
    let scope = use_hook(current_scope_id);
    let rest_at = use_signal(|| None);
    let exits = use_signal(Vec::new);
    let state = use_signal(|| RosterState::first_show(&keys, pitch));
    let roster = Roster {
        state,
        env,
        scope,
        rest_at,
        exits,
    };
    let mut seen = use_hook(|| {
        roster.schedule_rest();
        CopyValue::new(keys.clone())
    });
    if *seen.peek() != keys {
        let _ = roster.update(|state| state.reconcile(&keys));
        roster.schedule_rest();
        seen.set(keys);
    }
    roster
}
