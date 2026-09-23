//! AnimatedList: the `ul.ds-list` rows live in; its `data-presence` plays the first-show
//! entrance (design/04-COMPONENTS.md section 16 markup, the plan's wave-2 "animated_list").

use crate::motion::presence::ListPresence;
use dioxus::prelude::*;

/// The list's `data-presence` word.
fn presence_slug(presence: ListPresence) -> &'static str {
    match presence {
        ListPresence::Entering => "entering",
        ListPresence::Present => "present",
    }
}

/// A list of rows whose entrance plays only when first shown.
///
/// The rows are the consumer's `ListRow`s, one per `use_roster` entry, keyed by the roster key
/// so a leaving row keeps its node until it settles. While `presence` is `Entering` an entering
/// row rises staggered; once it is `Present` an entering row is an arrival and plays `row-in`.
#[component]
pub fn AnimatedList(label: String, presence: ListPresence, children: Element) -> Element {
    rsx! {
        ul {
            class: "ds-list",
            role: "listbox",
            "aria-label": "{label}",
            "data-presence": presence_slug(presence),
            {children}
        }
    }
}
