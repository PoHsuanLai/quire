//! Tooltip: text that names an action (Fly) or a value (Card) (design/04-COMPONENTS.md
//! section 18).

use dioxus::prelude::*;

/// Which tooltip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TooltipKind {
    /// A small dark label above a strip button, after `--d-fly` (0 when warm).
    Fly,
    /// A small hover card for a value, through the hover hub.
    Card,
}

/// A tooltip on `children`.
#[component]
pub fn Tooltip(
    kind: TooltipKind,
    text: String,
    #[props(default)] sub: Option<String>,
    children: Element,
) -> Element {
    todo!()
}
