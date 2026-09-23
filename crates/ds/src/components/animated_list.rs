//! AnimatedList: the `ul.ds-list` rows live in; its `data-presence` plays the first-show
//! entrance (design/04-COMPONENTS.md section 16 markup, the plan's wave-2 "animated_list").

use crate::motion::presence::ListPresence;
use dioxus::prelude::*;

/// A list of rows whose entrance plays only when first shown.
#[component]
pub fn AnimatedList(label: String, presence: ListPresence, children: Element) -> Element {
    todo!()
}
