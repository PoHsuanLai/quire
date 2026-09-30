//! The operation behind `Availability::Busy` (design/30 section 2.9): a control that is busy
//! shows a spinner, and a spinner needs an [`Operation`] to turn. The control has no service to
//! ask, so it starts one when it becomes busy and ends it when it stops.

use dioxus::core::queue_effect;
use dioxus::prelude::*;
use ds_core::vocab::Availability;
use ds_motion::detail::operation::{Operation, PendingToken};

/// The operation `availability` stands for: running from the render the control turns `Busy`,
/// idle again the moment it does not.
pub(crate) fn use_busy(availability: Availability) -> Operation {
    let mut token = use_signal(|| None::<PendingToken>);
    match (availability, *token.peek()) {
        (Availability::Busy, Some(running)) => Operation::Running(running),
        (Availability::Busy, None) => {
            let started = PendingToken::start();
            queue_effect(move || token.set(Some(started)));
            Operation::Running(started)
        }
        (Availability::Enabled | Availability::Disabled, Some(_)) => {
            queue_effect(move || token.set(None));
            Operation::Idle
        }
        (Availability::Enabled | Availability::Disabled, None) => Operation::Idle,
    }
}
