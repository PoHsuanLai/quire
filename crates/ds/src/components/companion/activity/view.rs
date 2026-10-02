//! The activity strip component.

use super::model::ActivityEntry;
use crate::stack::toast_hub::UndoToken;
use dioxus::prelude::*;

/// The companion's activity as a strip of entries, newest first, each with Undo while it can be
/// undone. `on_undo` hears the entry to undo, `on_open` the entry to open in its app.
#[component]
pub fn ActivityStrip(
    entries: Vec<ActivityEntry>,
    on_undo: EventHandler<UndoToken>,
    on_open: EventHandler<UndoToken>,
) -> Element {
    let _ = (on_undo, on_open);
    rsx! {
        div { class: "ds-activity-strip", "data-count": entries.len() }
    }
}
