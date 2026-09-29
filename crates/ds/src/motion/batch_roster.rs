//! A roster whose rows leave in batches when the consumer stops listing them: the
//! `LeavingList` lifecycle. Every key that drops off the list in one render is one batch: its
//! rows play the exit together, staggered in list order (capped at 12), and when the last of
//! them has settled they are dropped at once and the rows below heal by the sum of the heights
//! the dropped rows above each one measured (`RosterState::settled_batch`). A Clear is one
//! batch; a single dismissal is a batch of one; a group collapsing is the batch of the rows it
//! hides. A key listed again while its batch plays stays where it is and is not dropped.
//!
//! The batch's timer is its own task, not the per-key exit timer of `use_leaving_roster`:
//! keeping one row of a batch must not cancel the others. So a batch remembers which keys it
//! claimed, and a key that has left again since, in a later batch, is that batch's to drop.

use super::presence::{Exit, ListPresence, Presence};
use super::roster::{RosterState, RowPitch};
use super::roster_exits::Pitches;
use super::settle::settle;
use super::use_roster::{Roster, use_roster_parts};
use crate::core::task::{Gone, spawn_in, try_get};
use crate::core::time::clock::sleep;
use crate::core::vocab::{Emphasis, StaggerIndex};
use crate::motion::anim::Anim;
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// How a batch roster's rows come and go.
pub(crate) struct Departures<K: 'static> {
    /// The exit a batch plays, read on the render that starts it.
    pub(crate) exit: Exit,
    /// Whether the rows listed on the first render rise in (`Entering`) or are simply there.
    pub(crate) first: ListPresence,
    /// The heights the rows measured.
    pub(crate) pitches: Pitches<K>,
    /// Hears each dropped key once its batch has settled.
    pub(crate) on_settled: Option<EventHandler<K>>,
}

/// One batch's number, so a key that left again later is not dropped by an earlier batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BatchId(u64);

/// Which batch each leaving key belongs to, and the next batch's number.
#[derive(Debug, Clone, PartialEq)]
struct Claims<K> {
    held: Vec<(K, BatchId)>,
    next: BatchId,
}

impl<K: Clone + PartialEq> Claims<K> {
    /// Claim `keys` for a new batch, taking them from any earlier one.
    fn claim(&mut self, keys: &[K]) -> BatchId {
        let batch = self.next;
        self.next = BatchId(batch.0 + 1);
        self.held.retain(|(key, _)| !keys.contains(key));
        self.held
            .extend(keys.iter().map(|key| (key.clone(), batch)));
        batch
    }

    /// The keys `batch` still holds, released.
    fn release(&mut self, batch: BatchId) -> Vec<K> {
        let (mine, rest): (Vec<_>, Vec<_>) = self.held.drain(..).partition(|(_, b)| *b == batch);
        self.held = rest;
        mine.into_iter().map(|(key, _)| key).collect()
    }
}

/// A roster over `keys` whose missing keys leave in batches, and whether the list is still
/// playing its first show (`ListPresence::Entering` until the keys first change).
pub(crate) fn use_batch_roster<K: Clone + PartialEq + 'static>(
    keys: Vec<K>,
    departures: Departures<K>,
) -> (Roster<K>, ListPresence) {
    let roster = use_roster_parts(&keys, RowPitch::default());
    let mut claims = use_hook(|| {
        CopyValue::new(Claims {
            held: Vec::new(),
            next: BatchId(0),
        })
    });
    let first = departures.first;
    let mut shown = use_hook(|| {
        match first {
            ListPresence::Entering => roster.queue_rest(),
            ListPresence::Present => {
                let _ = roster.update(RosterState::rest);
            }
        }
        CopyValue::new(first)
    });
    let mut seen = use_hook(|| CopyValue::new(keys.clone()));
    if *seen.peek() == keys {
        return (roster, *shown.peek());
    }
    shown.set(ListPresence::Present);
    let before = seen.peek().clone();
    let gone: Vec<K> = before
        .iter()
        .filter(|key| !keys.contains(key))
        .cloned()
        .collect();
    for key in keys.iter().filter(|key| !before.contains(key)) {
        // Listed again while it leaves: it stays where it is (a no-op for a new key).
        let _ = roster.stay(key.clone());
    }
    let mut running = Vec::new();
    let _ = roster.update(|state| {
        let (state, started) = state.leave_batch(&gone, departures.exit, Emphasis::Plain);
        running = started;
        state.reconcile(&keys)
    });
    if !running.is_empty() {
        let batch = claims.write().claim(&gone);
        let timing = BatchTiming {
            batch,
            running,
            claims,
            pitches: departures.pitches,
            on_settled: departures.on_settled,
        };
        queue_effect(move || {
            let _ = roster.time_batch(timing);
        });
    }
    roster.queue_rest();
    seen.set(keys);
    (roster, ListPresence::Present)
}

/// What a batch's timer needs: which batch, the animations it waits for, and where the heights
/// and the listener are.
struct BatchTiming<K: 'static> {
    batch: BatchId,
    running: Vec<(Anim, StaggerIndex)>,
    claims: CopyValue<Claims<K>>,
    pitches: Pitches<K>,
    on_settled: Option<EventHandler<K>>,
}

impl<K: Clone + PartialEq + 'static> Roster<K> {
    /// Start a batch's timer: once its longest exit has settled, its rows still leaving are
    /// dropped together and the rows below heal by their measured heights.
    fn time_batch(&self, timing: BatchTiming<K>) -> Result<(), Gone> {
        let level = self.level()?;
        let length = timing
            .running
            .iter()
            .map(|&(anim, index)| settle(anim, level, index))
            .max()
            .unwrap_or_default();
        let roster = *self;
        spawn_in(self.scope, async move {
            sleep(length).await;
            let mut claims = timing.claims;
            let Ok(claimed) = claims
                .try_write()
                .map(|mut held| held.release(timing.batch))
            else {
                return;
            };
            // A key taken back meanwhile is present again: not dropped, not reported.
            let Ok(keys) = try_get(roster.state).map(|state| still_leaving(&state, claimed)) else {
                return;
            };
            let pitches = timing.pitches;
            let settled = roster.update(|state| {
                state.settled_batch(&keys, |key| pitches.of(key).unwrap_or_default())
            });
            if settled.is_err() {
                return;
            }
            roster.schedule_rest();
            for key in keys {
                pitches.forget(&key);
                if let Some(on_settled) = timing.on_settled {
                    on_settled.call(key);
                }
            }
        });
        Ok(())
    }
}

/// The keys among `keys` that `state` still shows leaving.
fn still_leaving<K: Clone + PartialEq>(state: &RosterState<K>, keys: Vec<K>) -> Vec<K> {
    keys.into_iter()
        .filter(|key| {
            state
                .entries()
                .iter()
                .any(|entry| &entry.key == key && matches!(entry.presence, Presence::Leaving(_)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{BatchId, Claims};

    #[test]
    fn a_later_batch_takes_a_key_from_an_earlier_one() {
        let mut claims = Claims {
            held: Vec::new(),
            next: BatchId(0),
        };
        let first = claims.claim(&[1, 2, 3]);
        let second = claims.claim(&[2]);
        assert_ne!(first, second);
        assert_eq!(claims.release(first), vec![1, 3]);
        assert_eq!(claims.release(second), vec![2]);
        assert_eq!(claims.release(first), Vec::<u32>::new());
    }
}
