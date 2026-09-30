//! What an operation on a settings row is doing, and where the row shows it (design/26-DETAILS.md
//! 5.2.2, 5.2.3, 5.2.8): joining a network, connecting a device, switching the
//! sound output.

use ds_motion::detail::{detailed::Detailed, moment::Moment, stamp::EventStamp};
use ds_core::word::Word;

/// An operation on the row's item, stamped by the service that runs it: the same stamp is the
/// same event, so a re-poll plays nothing and a repeated failure never shakes again (R6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RowPhase {
    /// Nothing running: the row as its trailing mark and disc say.
    #[default]
    Rest,
    /// Joining, connecting or switching to this row's item: a spinner takes the trailing slot.
    Pending(EventStamp),
    /// The operation ended as asked: the row is drawn as its trailing mark and disc say.
    Succeeded(EventStamp),
    /// It did not happen: put the reason in its detail line (R8).
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

/// The disc a row's glyph sits on: none (the bare glyph), the paper disc, or the accent disc of
/// the item in use (the connected network or device, design/26).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
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
    pub(crate) fn attr(self) -> Option<&'static str> {
        (self != RowDisc::None).then(|| self.slug())
    }
}
