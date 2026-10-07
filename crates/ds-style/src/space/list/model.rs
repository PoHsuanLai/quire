//! The state types: a Space, and the list of them.

use super::recall::Recall;
use crate::space::look::SpaceLook;
use serde::{Deserialize, Serialize};

/// A Space's stable identity. Positions change when Spaces are reordered or removed; an id does
/// not, so Today and Recall key on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SpaceId(pub u64);

/// One context: a name, how its frame looks, and whatever the app keeps in it.
///
/// Written flat: `id`, `name`, the look's own keys, then the payload's keys, so a payload that
/// is a struct sits beside the look in the file (and an app that kept those keys before the kit
/// reads its old file unchanged). A payload must not have keys called `id`, `name`, `dots`,
/// `grain`, `theme` or `card_accent`.
#[derive(Debug, Clone, PartialEq)]
pub struct Space<P> {
    /// Its identity.
    pub id: SpaceId,
    /// What the switcher calls it.
    pub name: String,
    /// The frame's dots, grain, theme and card accent.
    pub look: SpaceLook,
    /// The app's own.
    pub payload: P,
}

/// Every Space, which one is on screen, and where each was left.
///
/// Never empty: the last Space cannot be removed, and a file with none is not a `Spaces`.
/// `R` is the app's place type (`()` for an app that restores nothing).
#[derive(Debug, Clone, PartialEq)]
pub struct Spaces<P, R = ()> {
    pub(super) list: Vec<Space<P>>,
    pub(super) current: SpaceId,
    pub(super) recall: Recall<R>,
    pub(super) next: u64,
}

impl<P, R> Spaces<P, R> {
    /// The Spaces in order.
    pub fn list(&self) -> &[Space<P>] {
        &self.list
    }

    /// The Space on screen.
    pub fn current(&self) -> &Space<P> {
        self.get(self.current).unwrap_or(&self.list[0])
    }

    /// The Space with `id`.
    pub fn get(&self, id: SpaceId) -> Option<&Space<P>> {
        self.list.iter().find(|space| space.id == id)
    }

    /// Where `id` stands in the list.
    pub fn index_of(&self, id: SpaceId) -> Option<usize> {
        self.list.iter().position(|space| space.id == id)
    }

    /// The Space at `index`, as ⌘1-9 count them from zero.
    pub fn at(&self, index: usize) -> Option<&Space<P>> {
        self.list.get(index)
    }

    /// How many there are. At least one.
    pub fn count(&self) -> usize {
        self.list.len()
    }

    /// Where each Space was left.
    pub fn recall(&self) -> &Recall<R> {
        &self.recall
    }
}
