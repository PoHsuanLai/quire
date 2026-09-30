//! Spinner: an operation without a known end, drawn as a ring around its parent
//! (design/04-COMPONENTS.md section 15), played as a pending loop (design/26-DETAILS.md R4, design/30
//! section 1.3): it spins at once while the operation runs, in twelve steps of `--t-spin-step`
//! (a turn a second), and keeps turning under Reduced. It never loops without an [`Operation`].

use ds_motion::detail::{
    operation::Operation,
    pending::{PendingFrame, PendingLayers, PendingSpec, PendingStyle},
    use_pending::use_pending,
};
use dioxus::prelude::*;

/// The loop a spinner plays: one ring, a twelfth of a turn a step.
pub(crate) const SPIN: PendingSpec = PendingSpec {
    style: PendingStyle::Spin,
    layers: PendingLayers(1),
};

/// An activity ring for `operation`, drawn around its positioned parent (`inset:-4px`).
/// `data-pending` is `idle` (not shown) or `step` (moving: `--turn` is the ring's angle). A
/// standalone size is not specified (TODO(O-9)).
#[component]
pub fn Spinner(operation: Operation) -> Element {
    ring(use_pending(operation, SPIN))
}

/// The ring at `frame`: for a part that reads the frame itself (the preview pane, which also
/// swaps its media for the pending look while the loop shows).
pub(crate) fn ring(frame: PendingFrame) -> Element {
    let turn = match frame {
        PendingFrame::Step(n) => Some(format!("--turn:{}deg", u32::from(n) * 30)),
        PendingFrame::Idle => None,
    };
    rsx! {
        span {
            class: "ds-spinner",
            "data-pending": frame.slug(),
            style: turn,
            "aria-hidden": "true",
        }
    }
}
