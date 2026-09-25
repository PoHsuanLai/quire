//! A roster whose rows leave when the consumer stops listing them (sill Q121): the banner
//! stack's lifecycle. `use_roster` drops a key the consumer no longer lists at once, because a
//! mail row's exit is started by the action that removes it (`Roster::leave`); a notification
//! banner is simply taken off the caller's list when it times out or is dismissed, so here a
//! missing key plays `exit`, the rows below heal once it settles, each by the height the
//! leaving row measured (banners differ in height), and a key listed again while it leaves
//! stays.

use super::presence::Exit;
use super::roster::{RosterState, RowPitch};
use super::settle::settle;
use super::use_roster::{ExitTimer, Roster, use_roster_parts};
use crate::components::vocab::{Emphasis, StaggerIndex};
use crate::task::{Gone, spawn_in, try_get, try_set};
use crate::time::sleep;
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// The heights rows measured, by key: how far the rows below heal when one leaves.
#[derive(Debug, PartialEq)]
pub(crate) struct Pitches<K: 'static>(CopyValue<Vec<(K, RowPitch)>>);

impl<K: 'static> Clone for Pitches<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K: 'static> Copy for Pitches<K> {}

impl<K: Clone + PartialEq + 'static> Pitches<K> {
    /// Record `key`'s pitch.
    pub(crate) fn set(&self, key: K, pitch: RowPitch) {
        let mut book = self.0;
        let _ = book.try_write().map(|mut book| {
            book.retain(|(held, _)| *held != key);
            book.push((key, pitch));
        });
    }

    /// `key`'s pitch, if it measured one.
    fn of(&self, key: &K) -> Option<RowPitch> {
        self.0.try_peek().ok().and_then(|book| {
            book.iter()
                .find(|(held, _)| held == key)
                .map(|(_, pitch)| *pitch)
        })
    }

    /// Forget `key`: its row has gone.
    fn forget(&self, key: &K) {
        let mut book = self.0;
        let _ = book
            .try_write()
            .map(|mut book| book.retain(|(held, _)| held != key));
    }
}

/// An empty book of pitches.
pub(crate) fn use_pitches<K: 'static>() -> Pitches<K> {
    Pitches(use_hook(|| CopyValue::new(Vec::new())))
}

/// How a leaving roster behaves: the exit its rows play, the pitch a row that measured none
/// heals by, the book of measured pitches, and who hears a row's exit settle.
pub(crate) struct Leaving<K: 'static> {
    pub(crate) exit: Exit,
    pub(crate) pitch: RowPitch,
    pub(crate) pitches: Pitches<K>,
    pub(crate) on_settled: EventHandler<K>,
}

/// A roster over `keys` whose missing keys leave by `leaving.exit`.
pub(crate) fn use_leaving_roster<K: Clone + PartialEq + 'static>(
    keys: Vec<K>,
    leaving: Leaving<K>,
) -> Roster<K> {
    let roster = use_roster_parts(&keys, leaving.pitch);
    let mut seen = use_hook(|| {
        roster.queue_rest();
        CopyValue::new(keys.clone())
    });
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
    let exit = leaving.exit;
    let _ = roster.update(|state| {
        gone.iter()
            .fold(state, |state, key| {
                state.leave(key, exit, Emphasis::Plain).0
            })
            .reconcile(&keys)
    });
    for key in gone {
        let settled = leaving.on_settled;
        let (pitch, pitches) = (leaving.pitch, leaving.pitches);
        queue_effect(move || {
            let _ = roster.time_exit(key, exit, Heal { pitch, pitches }, settled);
        });
    }
    roster.queue_rest();
    seen.set(keys);
    roster
}

/// How a settled row's neighbours heal: by its measured pitch, else by `pitch`.
#[derive(Clone, Copy)]
struct Heal<K: 'static> {
    pitch: RowPitch,
    pitches: Pitches<K>,
}

impl<K: Clone + PartialEq + 'static> Roster<K> {
    /// Start `key`'s exit timer: at `settle(exit)` the row is dropped, the rows below heal by
    /// its pitch, and `settled` hears the key.
    fn time_exit(
        &self,
        key: K,
        exit: Exit,
        heal: Heal<K>,
        settled: EventHandler<K>,
    ) -> Result<(), Gone> {
        let anim = super::roster::exit_anim(exit, Emphasis::Plain);
        let length = settle(anim, self.level()?, StaggerIndex::default());
        self.cancel_exit(&key)?;
        let roster = *self;
        let settling = key.clone();
        let task = spawn_in(self.scope, async move {
            sleep(length).await;
            let _ = roster.forget_exit(&settling);
            let pitch = heal.pitches.of(&settling).unwrap_or(heal.pitch);
            heal.pitches.forget(&settling);
            if roster
                .update(|state: RosterState<K>| state.settled_by(&settling, pitch))
                .is_ok()
            {
                roster.schedule_rest();
                settled.call(settling);
            }
        });
        let mut exits = try_get(self.exits)?;
        exits.push(ExitTimer { key, task });
        try_set(self.exits, exits)
    }
}
