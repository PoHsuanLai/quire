//! The roster's rest timer: once every entering and healing row's animation has settled, the
//! rows are marked present.
//!
//! A reconcile happens in the component body (the rows to draw must be right in the render
//! that lists them), but the timer it needs is never spawned there: under the webview a task
//! spawned from a render may never be polled (mailo FINDINGS F140), so the body only queues an
//! effect, and the effect, which dioxus runs after the render on every renderer, spawns the
//! timer as a task of the roster's owner (`crate::task::spawn_in`, sill FINDINGS Q45). Two
//! reconciles before the effect runs queue it once, and a timer already due later than the new
//! one is kept rather than joined by a second task.

use super::roster::RosterState;
use super::settle::settle;
use super::use_roster::Roster;
use crate::task::{Gone, spawn_in, try_get, try_set};
use crate::time::sleep;
use dioxus::core::{Task, queue_effect};
use dioxus::prelude::*;
use std::time::Instant;

/// Whether a rest is waiting for the effect that schedules it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RestQueue {
    /// Nothing queued.
    Idle,
    /// An effect will schedule the rest after this render.
    Queued,
}

/// The one pending rest: when it is due, and the task that will run it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RestTimer {
    due: Instant,
    task: Task,
}

impl<K: Clone + PartialEq + 'static> Roster<K> {
    /// From a render: schedule the rest once the render is done, at most once per render
    /// batch.
    pub(super) fn queue_rest(&self) {
        let mut queue = self.rest_queue;
        if queue.try_peek().map(|queued| *queued) != Ok(RestQueue::Idle) {
            return;
        }
        queue.set(RestQueue::Queued);
        let roster = *self;
        queue_effect(move || {
            let mut queue = roster.rest_queue;
            if queue
                .try_write()
                .map(|mut queued| *queued = RestQueue::Idle)
                .is_ok()
            {
                roster.schedule_rest();
            }
        });
    }

    /// From a handler, a task or an effect: mark the entering and healing rows present once the
    /// longest of their animations has settled. A pending rest due at or after that is kept; an
    /// earlier one is cancelled and replaced, so there is never more than one rest task.
    pub(super) fn schedule_rest(&self) {
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
        if let Some(pending) = try_get(self.rest)? {
            if pending.due >= due {
                return Ok(());
            }
            pending.task.cancel();
        }
        let roster = *self;
        let task = spawn_in(self.scope, async move {
            sleep(due.saturating_duration_since(Instant::now())).await;
            if try_set(roster.rest, None).is_ok() {
                let _ = roster.update(RosterState::rest);
            }
        });
        #[cfg(test)]
        tests::SPAWNED.with(|spawned| spawned.set(spawned.get() + 1));
        try_set(self.rest, Some(RestTimer { due, task }))
    }
}

#[cfg(test)]
mod tests;
