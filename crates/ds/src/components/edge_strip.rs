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

impl SideState {
    /// The `data-side` value the consumer writes on its `.ds-side` container.
    pub fn slug(self) -> &'static str {
        match self {
            SideState::Shown => "shown",
            SideState::Hidden => "hidden",
            SideState::Peek => "peek",
        }
    }
}

/// The reveal edge. Render it only while the sidebar is hidden; the pointer entering it asks
/// for the peek (design/06-INTERACTIONS.md section 7).
#[component]
pub fn EdgeStrip(onenter: EventHandler<()>) -> Element {
    rsx! {
        div {
            class: "ds-edge",
            "aria-hidden": "true",
            onpointerenter: move |_| onenter.call(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SideState;

    #[test]
    fn every_state_the_stylesheet_moves_has_its_slug() {
        let css = include_str!("edge_strip.css");
        for state in [SideState::Hidden, SideState::Peek] {
            let selector = format!(".ds-side[*|data-side={}]", state.slug());
            assert!(css.contains(&selector), "{state:?}: no {selector}");
        }
        assert_eq!(SideState::Shown.slug(), "shown");
    }
}
