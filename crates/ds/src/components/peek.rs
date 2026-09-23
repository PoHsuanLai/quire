//! Peek: a thread's reader in a panel over the card (design/04-COMPONENTS.md section 24).

use crate::appearance::PeekMode;
use dioxus::prelude::*;

/// A reader floating over the card.
#[component]
pub fn Peek(
    mode: PeekMode,
    label: String,
    onclose: EventHandler<()>,
    children: Element,
) -> Element {
    todo!()
}
