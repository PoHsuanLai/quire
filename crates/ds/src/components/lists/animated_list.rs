//! AnimatedList: the `ul.ds-list` rows live in (design/04-COMPONENTS.md section 16 markup).

use dioxus::prelude::*;

/// A list of rows.
///
/// The rows are the consumer's `ListRow`s, one per `use_roster` entry, keyed by the roster key
/// so a leaving row keeps its node until it settles.
#[component]
pub fn AnimatedList(label: String, children: Element) -> Element {
    rsx! {
        ul {
            class: "ds-list",
            role: "listbox",
            "aria-label": "{label}",
            {children}
        }
    }
}
