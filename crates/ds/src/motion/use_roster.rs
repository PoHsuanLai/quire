//! The roster as a hook: [`RosterState`] in a signal, with the settle timers started for it.
//! The timers belong to the hook's owner and drop with it; a timer that finds the roster gone
//! stops (`crate::task`, sill FINDINGS Q45).

use super::presence::Exit;
use super::roster::{RosterEntry, RosterState, RowPitch};
use super::settle::settle;
use crate::components::vocab::{Emphasis, StaggerIndex};
use crate::root::env::{Env, use_env_signal};
use crate::task::{Gone, spawn_in, try_get, try_set};
use crate::time::sleep;
use dioxus::core::current_scope_id;
use dioxus::prelude::*;
use std::time::Instant;

/// A live roster: read its entries in render, start exits from handlers.
#[derive(Debug, PartialEq)]
pub struct Roster<K: 'static> {
    state: Signal<RosterState<K>>,
    env: Signal<Env>,
    scope: ScopeId,
    rest_at: Signal<Option<Instant>>,
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
        let roster = *self;
        spawn_in(self.scope, async move {
            sleep(length).await;
            if roster.update(|state| state.settled(&key)).is_ok() {
                roster.schedule_rest();
            }
        });
        Ok(())
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
    let state = use_signal(|| RosterState::first_show(&keys, pitch));
    let roster = Roster {
        state,
        env,
        scope,
        rest_at,
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
