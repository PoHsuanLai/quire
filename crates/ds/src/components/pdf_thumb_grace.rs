//! The pending look's grace (design/26 R4) on the Pending primitive: a thumbnail whose page is
//! still being read is a Pending moment of its own reading state, the operation that moment is
//! runs under a token whose deadline is the grace, so `use_pending` shows nothing until
//! [`PDF_THUMB_GRACE`](super::pdf_thumb::PDF_THUMB_GRACE) has passed (a read that lands quickly
//! never flashes a placeholder) and then holds its still frame, the dimmed sheet, with no step
//! in between: one wake per read, and 0 frames however long the read takes.

use super::pdf_thumb::PDF_THUMB_GRACE;
use crate::detail::{
    detailed::Detailed,
    first_show::FirstShow,
    moment::Moment,
    operation::Deadline,
    pending::{Layers, PendingFrame, PendingSpec, PendingStyle},
    touch::Touch,
    use_detail::use_detail,
    use_operation::use_operation_within,
    use_pending::use_pending,
};

/// Whether the page is being read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Reading {
    /// It is: the grace runs.
    Yes,
    /// It arrived, or failed: no grace.
    No,
}

impl Detailed for Reading {
    fn moment(from: &Self, to: &Self) -> Moment {
        match (from, to) {
            (Reading::Yes, Reading::Yes) | (Reading::No, Reading::No) => Moment::Rest,
            (Reading::No, Reading::Yes) => Moment::Pending,
            (Reading::Yes, Reading::No) => Moment::Change,
        }
    }

    fn first(state: &Self) -> Moment {
        match state {
            Reading::Yes => Moment::Pending,
            Reading::No => Moment::Rest,
        }
    }
}

/// Where a read stands against its grace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Grace {
    /// Younger than the grace, or not reading: show the state as it is.
    Within,
    /// Reading for longer than the grace: the pending look.
    Over,
}

/// The sheet's pending look: one still layer (the look never steps).
const SHEET: PendingSpec = PendingSpec {
    style: PendingStyle::Breathe,
    layers: Layers(1),
};

/// The grace of the current read, restarted each time `reading` turns to `Yes`.
pub(crate) fn use_grace(reading: Reading) -> Grace {
    let detail = use_detail(reading, FirstShow::Still, Touch::Remote);
    let operation = use_operation_within(detail.cue(), Deadline::within(PDF_THUMB_GRACE));
    match use_pending(operation, SHEET) {
        PendingFrame::Idle => Grace::Within,
        PendingFrame::Step(_) | PendingFrame::Stalled => Grace::Over,
    }
}

#[cfg(test)]
mod tests {
    use super::Reading::{No, Yes};
    use crate::detail::{
        detailed::{first_table, moment_table},
        moment::Moment,
    };

    #[test]
    fn a_read_is_pending_and_its_end_a_change() {
        moment_table(&[
            (No, Yes, Moment::Pending),
            (Yes, No, Moment::Change),
            (Yes, Yes, Moment::Rest),
            (No, No, Moment::Rest),
        ]);
        first_table(&[(Yes, Moment::Pending), (No, Moment::Rest)]);
    }
}
