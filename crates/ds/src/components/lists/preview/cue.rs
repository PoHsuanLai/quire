//! What the preview pane is told about its latest change (design/26-DETAILS.md
//! section 5.6): the caller's [`Cue`] from its own `use_detail`, or only who caused the showing,
//! and the pending look drawn over the media box while a load runs. Split from `preview_pane`,
//! which frames the content with the actions.
//!
//! The pane's entrance springs only on the person's own contact (R5): a showing a service or a
//! timer asked for slides in at `--e-out`. A Preview (the selection moved onto another kind), a
//! Change (a load landed) or a Failure (it did not) swaps the media in place with a `fade` at
//! `--t-quick` instead of replaying the entrance; under Reduced it snaps (R7).

use crate::components::controls::spinner::{SpinnerKind, ring};
use crate::components::overlays::shown_phase::Alias;
use crate::motion::detail::{
    cue::Cue,
    pending::{PendingFrame, PendingSpec},
    touch::Touch,
};
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

/// The `data-touch` word: which easing the entrance plays.
pub(crate) fn touch_slug(touch: Touch) -> &'static str {
    match touch {
        Touch::Contact(_) => "contact",
        Touch::Remote => "remote",
    }
}

/// The touch the entrance now playing was caused by: taken when an entrance starts (the shown
/// phase's alias flips) and kept until the next, so a later remote change cannot swap the easing
/// of an entrance a contact started, nor the other way round.
pub(crate) fn use_entrance_touch(alias: Alias, touch: Touch) -> Touch {
    let mut latched = use_hook(|| CopyValue::new((Alias::A, Touch::Remote)));
    let (seen, kept) = *latched.peek();
    if seen == alias {
        return kept;
    }
    latched.set((alias, touch));
    touch
}

/// The pending look's ring: the dashed one, a quarter turn per step.
pub(crate) const PANE_PENDING: SpinnerKind = SpinnerKind::Spin;

/// The loop [`PANE_PENDING`] plays.
pub(crate) fn pane_pending_spec() -> PendingSpec {
    PANE_PENDING.spec()
}

/// What the media box holds while a load runs past its grace: the ring (stepping, or its still
/// frame at the deadline and under Reduced) over the words that say so (R8).
pub(crate) fn pending_look(frame: PendingFrame) -> Element {
    rsx! {
        div { class: "ds-preview-pending", role: "status",
            span { class: "ds-preview-pending-mark", {ring(PANE_PENDING, frame)} }
            span { class: "ds-preview-pending-words", "Loading\u{2026}" }
        }
    }
}
