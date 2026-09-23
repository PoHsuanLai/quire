//! The roster as a hook: [`RosterState`] in a signal, with the settle timers started for it.

use super::presence::Exit;
use super::roster::{RosterEntry, RosterState, RowPitch};
use super::settle::settle;
use crate::components::vocab::{Emphasis, StaggerIndex};
use crate::root::env::{Env, use_env_signal};
use crate::time::{sleep, spawn_in};
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
        let (next, anim) = self.state.peek().clone().leave(&key, exit, emphasis);
        let mut state = self.state;
        state.set(next);
        let length = settle(anim, self.level(), StaggerIndex::default());
        let roster = *self;
        spawn_in(self.scope, async move {
            sleep(length).await;
            roster.update(|state| state.settled(&key));
            roster.schedule_rest();
        });
    }

    fn update(&self, step: impl FnOnce(RosterState<K>) -> RosterState<K>) {
        let next = step(self.state.peek().clone());
        let mut state = self.state;
        state.set(next);
    }

    fn level(&self) -> crate::appearance::MotionLevel {
        self.env.peek().resolved.motion
    }

    /// Mark the entering and healing rows present once the longest of their animations has
    /// settled. A later call that ends later supersedes an earlier one.
    fn schedule_rest(&self) {
        let level = self.level();
        let Some(length) = self
            .state
            .peek()
            .running()
            .into_iter()
            .map(|(anim, index)| settle(anim, level, index))
            .max()
        else {
            return;
        };
        let due = Instant::now() + length;
        let due = self.rest_at.peek().map_or(due, |pending| pending.max(due));
        let mut rest_at = self.rest_at;
        rest_at.set(Some(due));
        let roster = *self;
        spawn_in(self.scope, async move {
            sleep(due.saturating_duration_since(Instant::now())).await;
            if *rest_at.peek() == Some(due) {
                rest_at.set(None);
                roster.update(RosterState::rest);
            }
        });
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
        roster.update(|state| state.reconcile(&keys));
        roster.schedule_rest();
        seen.set(keys);
    }
    roster
}
