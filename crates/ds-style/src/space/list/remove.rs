//! Deleting a Space.

use super::model::{Space, SpaceId, Spaces};

/// Why a Space was not removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refused {
    /// The only Space left: there is always one.
    Last,
    /// No Space has that id.
    Missing,
}

/// What a removal did.
#[derive(Debug, Clone, PartialEq)]
pub struct Removed<P> {
    /// The Space that went, whole, so a caller can put it back.
    pub space: Space<P>,
    /// Where it was.
    pub index: usize,
    /// The Space on screen afterwards.
    pub now_current: SpaceId,
}

impl<P, R> Spaces<P, R> {
    /// Remove `id`. When it was on screen, the Space that slides into its place (else the new
    /// last) is. The app also calls `Today::drop_space` with the same id.
    pub fn remove(&mut self, id: SpaceId) -> Result<Removed<P>, Refused> {
        let index = self.index_of(id).ok_or(Refused::Missing)?;
        if self.list.len() <= 1 {
            return Err(Refused::Last);
        }
        let space = self.list.remove(index);
        self.recall.forget(id);
        if self.current == id {
            self.current = self.list[index.min(self.list.len() - 1)].id;
        }
        Ok(Removed {
            space,
            index,
            now_current: self.current,
        })
    }
}
