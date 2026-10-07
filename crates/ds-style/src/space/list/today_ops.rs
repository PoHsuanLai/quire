//! What Today does: open, close, clear, expire, park.

use super::model::SpaceId;
use super::time::{Epoch, IDLE};
use super::today::{Entry, Parked, Today};
use std::time::Duration;

impl<I: PartialEq, K: PartialEq> Today<I, K> {
    /// Insert `item` in `space`, or refresh it, and put it first.
    pub fn opened(&mut self, space: SpaceId, item: I, now: Epoch) {
        self.entries
            .retain(|entry| entry.space != space || entry.item != item);
        self.entries.insert(
            0,
            Entry {
                space,
                item,
                last_opened: now,
            },
        );
    }

    /// Remove the shortcut for `item` in `space`. The app's data stays where it is.
    pub fn close(&mut self, space: SpaceId, item: &I) {
        self.entries
            .retain(|entry| entry.space != space || entry.item != *item);
    }

    /// Put `thing` aside in `space`, or move it to the front with a new title.
    pub fn park(&mut self, space: SpaceId, thing: K, title: &str, now: Epoch) {
        self.parked.retain(|parked| parked.item != thing);
        self.parked.insert(
            0,
            Parked {
                space,
                item: thing,
                title: title.to_owned(),
                parked: now,
            },
        );
    }

    /// `thing` is open again, sent or gone: its entry goes, in every Space.
    pub fn unpark(&mut self, thing: &K) {
        self.parked.retain(|parked| parked.item != *thing);
    }
}

impl<I, K> Today<I, K> {
    /// Remove every shortcut in `space`.
    pub fn clear(&mut self, space: SpaceId) {
        self.entries.retain(|entry| entry.space != space);
    }

    /// `space` was deleted: forget its shortcuts and what it parked. Ids are stable, so no other
    /// Space's entries move.
    pub fn drop_space(&mut self, space: SpaceId) {
        self.entries.retain(|entry| entry.space != space);
        self.parked.retain(|parked| parked.space != space);
    }

    /// Drop every entry idle for more than [`IDLE`], and say how many went. Parked things stay.
    pub fn prune(&mut self, now: Epoch) -> usize {
        let before = self.entries.len();
        self.entries.retain(|entry| left(entry, now).is_some());
        before - self.entries.len()
    }

    /// The entries of `space` that still have time, newest first, each with what it has left.
    pub fn live(&self, space: SpaceId, now: Epoch) -> Vec<(&Entry<I>, Duration)> {
        self.entries
            .iter()
            .filter(|entry| entry.space == space)
            .filter_map(|entry| left(entry, now).map(|time| (entry, time)))
            .collect()
    }

    /// What `space` parked, newest first.
    pub fn parked_in(&self, space: SpaceId) -> Vec<&Parked<K>> {
        self.parked
            .iter()
            .filter(|parked| parked.space == space)
            .collect()
    }
}

/// How long `entry` has left at `now`, or `None` when it has been idle for more than [`IDLE`].
fn left<I>(entry: &Entry<I>, now: Epoch) -> Option<Duration> {
    let idle = now.since(entry.last_opened).unwrap_or_default();
    IDLE.checked_sub(idle)
}
