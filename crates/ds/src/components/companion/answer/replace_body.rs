//! A replacement: the text as it is and as it would become, then what became of the proposal.

use super::action::{ActionRole, CardAction};
use super::built_in::built_in;
use crate::components::companion::replace::model::{ReplacePhase, ReplaceProposal};
use dioxus::prelude::*;
use ds_core::vocab::EffectMark;

/// The id of the button that applies the proposal.
pub const APPLY: &str = "replace.apply";
/// The id of the button that drops the proposal.
pub const DISCARD: &str = "replace.discard";
/// The id of the button that puts the original back.
pub const UNDO: &str = "replace.undo";

/// What became of the proposal, in a few words.
fn phase_words(phase: ReplacePhase) -> &'static str {
    match phase {
        ReplacePhase::Proposed => "Nothing has changed yet",
        ReplacePhase::Applied { .. } => "Applied",
        ReplacePhase::Undone => "Undone",
        ReplacePhase::Discarded => "Discarded",
    }
}

/// The buttons the phase offers: Apply waits for the person; Undo is there while it can be.
pub fn replace_actions(phase: ReplacePhase) -> Vec<CardAction> {
    match phase {
        ReplacePhase::Proposed => vec![
            built_in(
                APPLY,
                "Apply",
                ActionRole::Primary,
                EffectMark::UndoableWrite,
            ),
            built_in(DISCARD, "Discard", ActionRole::Secondary, EffectMark::Read),
        ],
        ReplacePhase::Applied { .. } => {
            vec![built_in(
                UNDO,
                "Undo",
                ActionRole::Secondary,
                EffectMark::UndoableWrite,
            )]
        }
        ReplacePhase::Undone | ReplacePhase::Discarded => Vec::new(),
    }
}

/// The replacement as the card body.
#[component]
pub fn ReplaceBody(proposal: ReplaceProposal) -> Element {
    rsx! {
        div { class: "ds-card-replace",
            div { class: "ds-card-replace-side", "data-side": "original",
                span { class: "ds-card-label", "Now" }
                p { class: "ds-card-line", "{proposal.original}" }
            }
            div { class: "ds-card-replace-side", "data-side": "proposed",
                span { class: "ds-card-label", "Instead" }
                p { class: "ds-card-line", "{proposal.proposed}" }
            }
            span { class: "ds-card-replace-phase", "{phase_words(proposal.phase)}" }
        }
    }
}
