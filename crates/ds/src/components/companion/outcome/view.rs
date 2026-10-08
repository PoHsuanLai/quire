//! The unseen-outcome mark component.

use super::model::Outcome;
use dioxus::prelude::*;
use ds_core::word::Word;

/// The words a screen reader says for `outcome`.
fn spoken(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Done => "Finished, not yet seen",
        Outcome::Failed => "Failed, not yet seen",
    }
}

/// A still dot in the `ok` or `danger` tone, drawn nothing for `None`. It does not animate. The
/// orb and the run row draw it themselves; a consumer draws one only beside its own marks.
#[component]
pub fn OutcomeMark(outcome: Option<Outcome>) -> Element {
    let Some(outcome) = outcome else {
        return rsx! {};
    };
    rsx! {
        span {
            class: "ds-outcome-mark",
            "data-outcome": outcome.slug(),
            role: "img",
            "aria-label": spoken(outcome),
        }
    }
}
