//! What the preview pane is told about its latest change (design/26-DETAILS.md
//! section 5.6): the caller's [`Cue`] from its own `use_detail`, or only who caused the showing,
//! and the pending look drawn over the media box while a load runs. Split from `preview_pane`,
//! which frames the content with the actions.
//!
//! The pane's entrance slides in at `--e-out`. A Preview (the selection moved onto another kind),
//! a Change (a load landed) or a Failure (it did not) swaps the media in place with a `fade` at
//! `--t-quick` instead of replaying the entrance; under Reduced it snaps (R7).

use crate::components::controls::spinner::ring;
use crate::motion::detail::{cue::Cue, pending::PendingFrame, touch::Touch};
use dioxus::prelude::*;

/// What a [`PreviewPane`](crate::PreviewPane) knows of its latest change: the cue the caller's
/// own `use_detail` made for the pane's state (its entrance's touch, and the in-place changes it
/// cross-fades), or only the touch that showed it (no cross-fades). Either converts in with
/// `.into()`; the default is a remote showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaneCue {
    /// The caller's cue for the pane's state.
    Cue(Cue),
    /// Only who caused the showing.
    Touch(Touch),
}

impl Default for PaneCue {
    /// A showing nobody touched: the entrance settles at `--e-out`.
    fn default() -> Self {
        PaneCue::Touch(Touch::Remote)
    }
}

impl From<Cue> for PaneCue {
    fn from(cue: Cue) -> Self {
        PaneCue::Cue(cue)
    }
}

impl From<Touch> for PaneCue {
    fn from(touch: Touch) -> Self {
        PaneCue::Touch(touch)
    }
}

impl PaneCue {
    /// Who caused the latest change.
    pub fn touch(self) -> Touch {
        match self {
            PaneCue::Cue(cue) => cue.touch(),
            PaneCue::Touch(touch) => touch,
        }
    }

    /// The cue, when the caller gave one.
    pub fn cue(self) -> Option<Cue> {
        match self {
            PaneCue::Cue(cue) => Some(cue),
            PaneCue::Touch(_) => None,
        }
    }
}

/// What the media box holds while a load runs: the ring, stepping, over the words that say so
/// (R8).
pub(crate) fn pending_look(frame: PendingFrame) -> Element {
    rsx! {
        div { class: "ds-preview-pending", role: "status",
            span { class: "ds-preview-pending-mark", {ring(frame)} }
            span { class: "ds-preview-pending-words", "Loading\u{2026}" }
        }
    }
}
