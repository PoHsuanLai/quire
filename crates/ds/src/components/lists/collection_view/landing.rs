//! A tile as a place that takes a drop: the pointer hooks the owner hears for a tile whose
//! `CollectionItem::drop` says it accepts, as a `Row` hands its pointer to the caller.

use dioxus::prelude::EventHandler;
use ds_core::vocab::DropState;

/// Where the pointer over an accepting tile is reported, each with the tile's index.
#[derive(Clone, Copy)]
pub(crate) struct Drops {
    /// The pointer came over the tile.
    pub(crate) enter: Option<EventHandler<usize>>,
    /// The pointer left the tile.
    pub(crate) leave: Option<EventHandler<usize>>,
    /// The pointer was released on the tile: the drop.
    pub(crate) drop: Option<EventHandler<usize>>,
}

/// An accepting tile at index `at`.
#[derive(Clone, Copy)]
pub(crate) struct Landing {
    /// The tile's index.
    pub(crate) at: usize,
    /// Where the pointer is reported.
    pub(crate) drops: Drops,
    /// Whether the owner lit the tile as the drop's target.
    state: DropState,
}

impl Landing {
    /// `at` as a place that takes a drop, or `None` when `state` is not one that does (`Idle`,
    /// or the `Source` being dragged).
    pub(crate) fn of(at: usize, state: DropState, drops: Drops) -> Option<Landing> {
        match state {
            DropState::Accepts | DropState::Target => Some(Landing { at, drops, state }),
            DropState::Idle | DropState::Source => None,
        }
    }

    /// The pointer came over the tile.
    pub(crate) fn enter(self) {
        if let Some(hear) = self.drops.enter {
            hear.call(self.at);
        }
    }

    /// The pointer left the tile.
    pub(crate) fn leave(self) {
        if let Some(hear) = self.drops.leave {
            hear.call(self.at);
        }
    }

    /// The pointer was released on the tile: a drop only on the tile the owner lit as the
    /// target, so a plain click on a folder that accepts drops is not one.
    pub(crate) fn release(self) {
        if let (DropState::Target, Some(hear)) = (self.state, self.drops.drop) {
            hear.call(self.at);
        }
    }

    /// Whether the tile is the lit target, whose release is the drop: Blitz follows that release
    /// with a click, which must not also pick the tile.
    pub(crate) fn lit(self) -> bool {
        matches!(self.state, DropState::Target)
    }
}
