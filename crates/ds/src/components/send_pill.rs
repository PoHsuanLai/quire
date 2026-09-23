//! SendPill: undo send, a countdown ring and an Undo, then "Sent" (design/04-COMPONENTS.md
//! section 31).

use crate::components::vocab::Fraction;
use dioxus::prelude::*;

/// Where the send is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SendPhase {
    /// Counting down; Undo is offered.
    Counting,
    /// Sent; Undo is gone.
    Done,
}

/// The undo-send pill.
#[component]
pub fn SendPill(
    text: String,
    progress: Fraction,
    phase: SendPhase,
    onundo: EventHandler<()>,
) -> Element {
    todo!()
}
