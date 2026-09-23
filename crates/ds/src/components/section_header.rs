//! SectionHeader: a small-caps label that names a group (design/04-COMPONENTS.md section 13).

use dioxus::prelude::*;

/// Where the header sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeaderKind {
    /// Sidebar groups on the frame, with a trailing rule and an optional action.
    Frame,
    /// List groups on the card, with a count and a trailing rule.
    Group,
    /// A label above a control, with an optional value.
    Field,
    /// A group title inside a menu or palette.
    Menu,
}

/// A group's name.
#[component]
pub fn SectionHeader(
    kind: HeaderKind,
    text: String,
    #[props(default)] value: Option<String>,
    #[props(default)] action: Option<(String, EventHandler<()>)>,
) -> Element {
    todo!()
}
