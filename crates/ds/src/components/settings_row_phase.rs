//! What an operation on a settings row is doing, and where the row shows it (design/26-DETAILS.md
//! 5.2.2, 5.2.3, 5.2.8): joining a network, connecting a device, switching the
//! sound output.

use crate::detail::{
    detailed::Detailed,
    moment::Moment,
    pending::{Layers, PendingSpec, PendingStyle},
    stamp::EventStamp,
};

/// An operation on the row's item, stamped by the service that runs it: the same stamp is the
/// same event, so a re-poll plays nothing and a repeated failure never shakes again (R6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RowPhase {
    /// Nothing running: the row as its trailing mark and disc say.
    #[default]
    Rest,
    /// Joining, connecting or switching to this row's item: a bounded pending loop after
    /// `PendingGrace` (where [`RowWork`] says), held still at `PendingCap` (R4).
    Pending(EventStamp),
    /// The operation ended as asked: the disc seals (`Settle{LockIn}`) or the check draws on
    /// (`Settle{Check}`) once, if the pending loop was watched; then the row rests as it is.
    Succeeded(EventStamp),
    /// It did not happen: the row shakes once; put the reason in its detail line (R8).
    Failed(EventStamp),
}

impl Detailed for RowPhase {
    fn moment(from: &Self, to: &Self) -> Moment {
        match (from, to) {
            (
                RowPhase::Rest
                | RowPhase::Pending(_)
                | RowPhase::Succeeded(_)
                | RowPhase::Failed(_),
                RowPhase::Pending(_),
            ) => Moment::Pending,
            (RowPhase::Pending(_), RowPhase::Succeeded(_)) => Moment::Success,
            (
                RowPhase::Rest
                | RowPhase::Pending(_)
                | RowPhase::Succeeded(_)
                | RowPhase::Failed(_),
                RowPhase::Failed(_),
            ) => Moment::Failure,
            // A success nobody watched start, or the row going back to rest: no ceremony.
            (
                RowPhase::Rest | RowPhase::Succeeded(_) | RowPhase::Failed(_),
                RowPhase::Succeeded(_),
            )
            | (
                RowPhase::Rest
                | RowPhase::Pending(_)
                | RowPhase::Succeeded(_)
                | RowPhase::Failed(_),
                RowPhase::Rest,
            ) => Moment::Change,
        }
    }

    /// A row shown mid-operation is an operation already running; anything else is simply there
    /// (a pane of rows has no Appear of its own: design/26 5.2.2).
    fn first(state: &Self) -> Moment {
        match state {
            RowPhase::Pending(_) => Moment::Pending,
            RowPhase::Rest | RowPhase::Succeeded(_) | RowPhase::Failed(_) => Moment::Rest,
        }
    }
}

/// Where a row's pending loop shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RowWork {
    /// The glyph breathes (`Pending{Breathe}`): a device connecting, an output switching.
    #[default]
    Glyph,
    /// A small spinner in the trailing slot, in place of the mark (`Pending{Spin}`): the network
    /// being joined, where the lock was.
    Trailing,
}

impl RowWork {
    /// The loop it plays.
    pub(crate) fn spec(self) -> PendingSpec {
        let style = match self {
            RowWork::Glyph => PendingStyle::Breathe,
            RowWork::Trailing => PendingStyle::Spin,
        };
        PendingSpec {
            style,
            layers: Layers(1),
        }
    }
}

/// The disc a row's glyph sits on: none (the bare glyph), the paper disc, or the accent disc of
/// the item in use (the connected network or device, design/26).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RowDisc {
    /// The bare glyph.
    #[default]
    None,
    /// A paper disc with ink: an item not in use.
    Off,
    /// An `--accent` disc with `--accent-ink`: the item in use.
    On,
}

impl RowDisc {
    /// The `data-disc` word.
    pub(crate) fn slug(self) -> Option<&'static str> {
        match self {
            RowDisc::None => None,
            RowDisc::Off => Some("off"),
            RowDisc::On => Some("on"),
        }
    }
}
