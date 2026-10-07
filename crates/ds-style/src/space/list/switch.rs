//! Switching Space: the decision, on plain values.

use super::model::{SpaceId, Spaces};

/// Which edge the new Space's content comes in from: from the right when the Space is later in
/// the list, from the left when it is earlier (design/21-SPACES.md section 13: `in-r`, `in-l`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlideIn {
    /// Moving to a later Space.
    Right,
    /// Moving to an earlier Space.
    Left,
}

impl SlideIn {
    /// The word a stylesheet keys the slide on: `in-r` or `in-l`.
    pub fn attribute(self) -> &'static str {
        match self {
            SlideIn::Right => "in-r",
            SlideIn::Left => "in-l",
        }
    }
}

/// What a switch did, for the app to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Switched<R> {
    /// The Space that was left.
    pub from: SpaceId,
    /// The Space now on screen.
    pub to: SpaceId,
    /// The edge the content slides in from.
    pub slide: SlideIn,
    /// Where `to` was left, for the app to show again.
    pub restore: R,
}

impl<P, R: Clone + Default> Spaces<P, R> {
    /// Leave the current Space, which the app was `leaving` at, for `to`. `None` when `to` is
    /// the current Space or does not exist; nothing changes then.
    pub fn switch_to(&mut self, to: SpaceId, leaving: R) -> Option<Switched<R>> {
        let from = self.current;
        let (from_at, to_at) = (self.index_of(from)?, self.index_of(to)?);
        if from == to {
            return None;
        }
        self.recall.leave(from, leaving);
        self.current = to;
        Some(Switched {
            from,
            to,
            slide: if to_at > from_at {
                SlideIn::Right
            } else {
                SlideIn::Left
            },
            restore: self.recall.of(to),
        })
    }

    /// [`Self::switch_to`] the Space at `index`: what ⌘1-9 ask for.
    pub fn switch_to_index(&mut self, index: usize, leaving: R) -> Option<Switched<R>> {
        let to = self.at(index)?.id;
        self.switch_to(to, leaving)
    }
}
