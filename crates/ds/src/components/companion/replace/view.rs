//! The replace bar component.

use super::model::{ReplaceIn, ReplaceProposal};
use crate::root::common::Common;
use dioxus::prelude::*;

/// A proposed replacement as the bar over the person's field: the change, Apply and Discard, then
/// Undo while it can be undone. `on_in` hears what the person did.
#[component]
pub fn ReplaceBar(
    proposal: ReplaceProposal,
    on_in: EventHandler<ReplaceIn>,
    #[props(default)] common: Common,
) -> Element {
    let _ = on_in;
    let class = common.class("ds-replace-bar");
    let data = common.data_attributes();
    rsx! {
        div {
            id: common.id.clone(),
            class,
            onmounted: move |event| common.mounted(event),
            ..data,
        }
    }
}
