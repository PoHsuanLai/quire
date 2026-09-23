//! HoverCard: a preview that opens after the pointer rests, never marks read, never fetches
//! (design/04-COMPONENTS.md section 22, design/06-INTERACTIONS.md section 3).

use crate::overlay::hover_hub::{HoverKey, HoverKind};
use dioxus::prelude::*;

/// Wraps whatever a card hooks: feeds the hover hub.
///
/// The component doc names the first prop `key`; dioxus reserves `key` for list identity and
/// rejects a prop of that name, so it is `hover_key` (FINDINGS.md).
#[component]
pub fn HoverTarget(hover_key: HoverKey, kind: HoverKind, children: Element) -> Element {
    todo!()
}

/// The card, rendered by the consumer for the hub's open key.
#[component]
pub fn HoverCard(kind: HoverKind, children: Element) -> Element {
    todo!()
}
