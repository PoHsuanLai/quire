//! Where each Space was left.

use super::model::SpaceId;
use std::collections::BTreeMap;

/// The place an app showed in each Space when you left it, restored on the way back.
///
/// Beside the Spaces rather than inside one because it is where you are, not how the Space
/// looks: the editor's Esc restores a Space exactly and must not also move you. A Space with no
/// entry yields the place's default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recall<R>(pub(super) BTreeMap<SpaceId, R>);

impl<R> Default for Recall<R> {
    fn default() -> Self {
        Recall(BTreeMap::new())
    }
}

impl<R: Clone + Default> Recall<R> {
    /// Where `id` was left, or the default place.
    pub fn of(&self, id: SpaceId) -> R {
        self.0.get(&id).cloned().unwrap_or_default()
    }
}

impl<R> Recall<R> {
    /// `id` was left at `place`.
    pub fn leave(&mut self, id: SpaceId, place: R) {
        self.0.insert(id, place);
    }

    /// Change the place of `id` in place, from the default when it has none.
    pub fn edit(&mut self, id: SpaceId, change: impl FnOnce(&mut R))
    where
        R: Default,
    {
        change(self.0.entry(id).or_default());
    }

    /// Forget `id`.
    pub fn forget(&mut self, id: SpaceId) {
        self.0.remove(&id);
    }

    /// How many Spaces have a place.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether no Space has one.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
