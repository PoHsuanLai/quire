//! The memory timeline and the consolidation view.

use super::model::{ConsolidationDiff, MemoryDay};
use dioxus::prelude::*;
use ds_core::vocab::MemoryVerb;

/// The memory as a timeline of days. `on_verb` hears the key of the row and what the person did
/// with it (forget, keep, discard, open the source, export).
#[component]
pub fn MemoryTimeline(
    days: Vec<MemoryDay>,
    on_verb: EventHandler<(String, MemoryVerb)>,
) -> Element {
    let _ = on_verb;
    rsx! {
        div { class: "ds-memory-timeline", "data-days": days.len() }
    }
}

/// One night's consolidation as a diff the person can read.
#[component]
pub fn ConsolidationView(diff: ConsolidationDiff) -> Element {
    rsx! {
        div { class: "ds-consolidation-view", "data-lines": diff.lines.len() }
    }
}
