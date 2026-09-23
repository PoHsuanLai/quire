//! EdgeStrip: with the sidebar hidden, a strip on the left edge brings it back as a floating
//! panel (design/04-COMPONENTS.md section 33).

use dioxus::prelude::*;

/// The sidebar container's state: `data-side`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SideState {
    /// In the grid.
    #[default]
    Shown,
    /// Collapsed; the edge strip is present.
    Hidden,
    /// Floating over the card while the pointer is on it.
    Peek,
}

/// The reveal edge.
#[component]
pub fn EdgeStrip(onenter: EventHandler<()>) -> Element {
    todo!()
}
