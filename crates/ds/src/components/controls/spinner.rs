//! Spinner: an operation without a known end, drawn as a ring around its parent
//! (design/04-COMPONENTS.md section 15), played as a bounded pending loop (design/26-DETAILS.md
//! R4): nothing until the operation has run `PendingGrace`, then a step every `--t-pending-step`,
//! then its still frame from the operation's deadline (at most `PendingCap`) or at once under
//! Reduced. It never loops without an [`Operation`], and never past its deadline.

use crate::motion::detail::{
    operation::Operation,
    pending::{Layers, PendingFrame, PendingSpec, PendingStyle},
    use_pending::use_pending,
};
use dioxus::prelude::*;

/// Which activity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpinnerKind {
    /// Work in progress: a dashed ring turning a quarter per step.
    Spin,
    /// Waiting on something: the ring's opacity between .45 and 1, a half per step.
    Breathe,
}

impl SpinnerKind {
    /// The `data-kind` word.
    fn slug(self) -> &'static str {
        match self {
            SpinnerKind::Spin => "spin",
            SpinnerKind::Breathe => "breathe",
        }
    }

    /// The pending loop it plays.
    pub(crate) fn spec(self) -> PendingSpec {
        let style = match self {
            SpinnerKind::Spin => PendingStyle::Spin,
            SpinnerKind::Breathe => PendingStyle::Breathe,
        };
        PendingSpec {
            style,
            layers: Layers(1),
        }
    }
}

/// An activity ring for `operation`, drawn around its positioned parent (`inset:-4px`).
/// `data-pending` is `idle` (not shown), `step` (moving: `data-beat` is the half for Breathe, and
/// `--turn` the ring's angle for Spin) or `still` (the held frame). A standalone size is not
/// specified (TODO(O-9)).
#[component]
pub fn Spinner(kind: SpinnerKind, operation: Operation) -> Element {
    ring(kind, use_pending(operation, kind.spec()))
}

/// The ring `kind` draws at `frame`: for a part that reads the frame itself (the preview pane,
/// which also swaps its media for the pending look while the loop shows).
pub(crate) fn ring(kind: SpinnerKind, frame: PendingFrame) -> Element {
    let (beat, turn) = match frame {
        PendingFrame::Step(n) => (
            Some(beat(n)),
            Some(format!("--turn:{}deg", u32::from(n) * 90)),
        ),
        PendingFrame::Idle | PendingFrame::Stalled => (None, None),
    };
    let turn = turn.filter(|_| kind == SpinnerKind::Spin);
    rsx! {
        span {
            class: "ds-spinner",
            "data-kind": kind.slug(),
            "data-pending": frame.slug(),
            "data-beat": beat,
            style: turn,
            "aria-hidden": "true",
        }
    }
}

/// The half of a breath step `n` is: `high` then `low`.
fn beat(n: u8) -> &'static str {
    if n.is_multiple_of(2) { "high" } else { "low" }
}
