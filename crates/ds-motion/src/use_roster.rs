//! The roster as one hook: [`RosterState`] in a signal, with the settle timers started for it.
//! The timers belong to the hook's owner and drop with it; a timer that finds the roster gone
//! stops (`ds_style::task`).
//!
//! Rows leave in batches: every key that starts leaving in one go plays the exit together, and
//! when the exit has settled they are dropped at once and the rows below heal by the heights the
//! dropped rows measured. A key listed again while it leaves stays where it is.

use super::presence::Exit;
use super::roster::{RosterEntry, RosterState, RowPitch, StayError, Stayed};
use super::roster_rest::{RestQueue, RestTimer};
use super::settle::settle;
use dioxus::core::queue_effect;
use dioxus::prelude::*;
use ds_core::time::clock::sleep;
use ds_style::scope::{Scope, use_scope_signal};
use ds_style::task::{Gone, spawn_in, try_get};

/// How a row leaves the roster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LeaveBy {
    /// The consumer asks: [`Roster::leave`] plays the exit, and a key the consumer stops listing
    /// without asking is dropped at once.
    Action,
    /// The consumer stops listing the row: it plays the exit and stays drawn until it settles.
    Delist,
}

/// What a roster does with its rows.
#[derive(Debug)]
pub struct RosterSpec<K: 'static> {
    /// How a row leaves.
    pub leave: LeaveBy,
    /// The exit its rows play, read when a batch starts.
    pub exit: Exit,
    /// How far the rows below heal for a row that measured nothing.
    pub pitch: RowPitch,
    /// Hears each dropped key once its batch has settled.
    pub on_settled: Option<EventHandler<K>>,
}

impl<K: 'static> Clone for RosterSpec<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K: 'static> Copy for RosterSpec<K> {}

/// The heights rows measured, by key: how far the rows below heal when one leaves.
#[derive(Debug, PartialEq)]
pub struct Pitches<K: 'static>(CopyValue<Vec<(K, RowPitch)>>);

impl<K: 'static> Clone for Pitches<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K: 'static> Copy for Pitches<K> {}

impl<K: Clone + PartialEq + 'static> Pitches<K> {
    /// Record `key`'s pitch.
    pub fn set(&self, key: K, pitch: RowPitch) {
        let mut book = self.0;
        let _ = book.try_write().map(|mut book| {
            book.retain(|(held, _)| *held != key);
            book.push((key, pitch));
        });
    }

    fn of(&self, key: &K) -> Option<RowPitch> {
        self.0.try_peek().ok().and_then(|book| {
            book.iter()
                .find(|(held, _)| held == key)
                .map(|(_, pitch)| *pitch)
        })
    }

    fn forget(&self, key: &K) {
        let mut book = self.0;
        let _ = book
            .try_write()
            .map(|mut book| book.retain(|(held, _)| held != key));
    }
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

/// A live roster: read its entries in render, start exits from handlers.
#[derive(Debug, PartialEq)]
pub struct Roster<K: 'static> {
    pub(super) state: Signal<RosterState<K>>,
    env: Signal<Scope>,
    pub(super) scope: ScopeId,
    pub(super) rest: Signal<Option<RestTimer>>,
    pub(super) rest_queue: CopyValue<RestQueue>,
    spec: CopyValue<RosterSpec<K>>,
    claims: CopyValue<Claims<K>>,
    pitches: Pitches<K>,
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

    /// The book rows record their measured heights in, for the rows below to heal by.
    pub fn pitches(&self) -> Pitches<K> {
        self.pitches
    }

    /// Start `key`'s exit; it is dropped, and the rows below heal, when the exit settles.
    pub fn leave(&self, key: K) {
        let _ = self.start_batch(&[key]);
    }

    /// Take `key`'s exit back while it plays: the row is present again where it was and is not
    /// dropped when its batch settles, so nothing below it heals (an undo before the row was
    /// dropped). The consumer lists the key again in the same handler, so the next reconcile
    /// keeps the row. A row that is not leaving is unchanged; a key the roster no longer holds
    /// is [`StayError::UnknownKey`] (its exit settled: list it again and it enters).
    pub fn stay(&self, key: K) -> Result<Stayed, StayError> {
        let (next, stayed) = try_get(self.state)
            .map_err(|Gone| StayError::Unmounted)?
            .stay(&key);
        ds_style::task::try_set(self.state, next).map_err(|Gone| StayError::Unmounted)?;
        stayed
    }

    /// Start the exits of `keys` as one batch and time it.
    fn start_batch(&self, keys: &[K]) -> Result<(), Gone> {
        let spec = self.spec.try_peek().map(|held| *held).map_err(|_| Gone)?;
        let mut started = Vec::new();
        self.update(|state| {
            let (state, keys) = state.leave_batch(keys, spec.exit);
            started = keys;
            state
        })?;
        if started.is_empty() {
            return Ok(());
        }
        let mut claims = self.claims;
        let batch = claims.try_write().map_err(|_| Gone)?.claim(&started);
        let roster = *self;
        queue_effect(move || {
            let _ = roster.time_batch(batch);
        });
        Ok(())
    }

    /// Start a batch's timer: once its exit has settled, its rows still leaving are dropped
    /// together and the rows below heal by their measured heights.
    fn time_batch(&self, batch: BatchId) -> Result<(), Gone> {
        let spec = self.spec.try_peek().map(|held| *held).map_err(|_| Gone)?;
        let length = settle(spec.exit.anim(), self.level()?);
        let roster = *self;
        spawn_in(self.scope, async move {
            sleep(length).await;
            let mut claims = roster.claims;
            let Ok(claimed) = claims.try_write().map(|mut held| held.release(batch)) else {
                return;
            };
            // A key taken back meanwhile is present again: not dropped, not reported.
            let Ok(keys) = try_get(roster.state).map(|state| still_leaving(&state, claimed)) else {
                return;
            };
            let (pitches, fallback) = (roster.pitches, spec.pitch);
            let settled = roster.update(|state| {
                state.settled_batch(&keys, |key| pitches.of(key).unwrap_or(fallback))
            });
            if settled.is_err() {
                return;
            }
            roster.schedule_rest();
            for key in keys {
                pitches.forget(&key);
                if let Some(on_settled) = spec.on_settled {
                    on_settled.call(key);
                }
            }
        });
        Ok(())
    }

    pub(super) fn update(
        &self,
        step: impl FnOnce(RosterState<K>) -> RosterState<K>,
    ) -> Result<(), Gone> {
        let next = step(try_get(self.state)?);
        ds_style::task::try_set(self.state, next)
    }

    pub(super) fn level(&self) -> Result<ds_style::appearance::motion::MotionLevel, Gone> {
        Ok(try_get(self.env)?.resolved.motion)
    }
}

/// The keys among `keys` that `state` still shows leaving.
fn still_leaving<K: Clone + PartialEq>(state: &RosterState<K>, keys: Vec<K>) -> Vec<K> {
    keys.into_iter()
        .filter(|key| {
            state.entries().iter().any(|entry| {
                &entry.key == key && matches!(entry.presence, super::presence::Presence::Leaving(_))
            })
        })
        .collect()
}

/// A roster over `keys`, which the consumer passes on every render. The first render shows
/// every key present; after that a change in `keys` reconciles: new keys enter, and a key that
/// goes missing leaves as `spec.leave` says (keys missing at once are one batch). Reconciling is
/// pure and happens in the render; the timers it needs are started after the render, by an
/// effect, never from the body.
pub fn use_roster<K: Clone + PartialEq + 'static>(keys: Vec<K>, spec: RosterSpec<K>) -> Roster<K> {
    let roster = Roster {
        state: use_signal(|| RosterState::first_show(&keys)),
        env: use_scope_signal(),
        scope: use_hook(dioxus::core::current_scope_id),
        rest: use_signal(|| None),
        rest_queue: use_hook(|| CopyValue::new(RestQueue::Idle)),
        spec: use_hook(|| CopyValue::new(spec)),
        claims: use_hook(|| {
            CopyValue::new(Claims {
                held: Vec::new(),
                next: BatchId(0),
            })
        }),
        pitches: Pitches(use_hook(|| CopyValue::new(Vec::new()))),
    };
    let mut held = roster.spec;
    held.set(spec);
    let mut seen = use_hook(|| CopyValue::new(keys.clone()));
    if *seen.peek() == keys {
        return roster;
    }
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
    match spec.leave {
        LeaveBy::Delist => {
            let _ = roster.start_batch(&gone);
        }
        LeaveBy::Action => {}
    }
    let _ = roster.update(|state| state.reconcile(&keys));
    roster.queue_rest();
    seen.set(keys);
    roster
}
