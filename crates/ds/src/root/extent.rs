//! How tall and wide a root is: as its content (a window, the bar, a card) or the whole viewport
//! (an overlay surface).
//!
//! A `.ds` root is a block in the document's flow, so it is as tall as what it lays out. A root
//! whose content is all positioned (a centred `Sheet`, an OSD card, a click catcher) lays out
//! nothing and is 0 px tall; on Blitz so is everything above it (dioxus-native-dom's `#main` is
//! `height:auto`), and a `position:fixed; inset:0` child does not help, since Taffy places a
//! fixed box against its parent rather than the viewport. A `Viewport` root claims
//! the viewport's size itself, with `min-height:100vh; min-width:100vw`, so what it positions has
//! the surface's whole area to be placed in.
//!
//! A `Popup` root is the opposite: a popup document whose host fits the surface to its content.
//! Its overlay host lays the root's one floating card (a `Menu`, a `Popover`) in flow at the
//! root's origin instead of absolutely over the whole root, so the root, and the surface fitted
//! to it, is the card's size, and Blitz, which hit-tests an overlay by its own box, delivers a
//! press to the card. A submenu (`data-depth`) stays absolute beside its parent. The outside
//! catcher does not cover the surface: an outside press is the compositor's grab, not the
//! document's. Keyboard focus, Escape and the menu tracker are unchanged. The rules are the
//! stylesheet's (`popover.css`).

/// How big a root is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RootExtent {
    /// As big as its content, the document's usual rule: a window, the bar, a card.
    #[default]
    Content,
    /// At least the viewport: an overlay surface whose content is positioned (a sheet, an OSD,
    /// a catcher). Written `data-extent="viewport"`.
    Viewport,
    /// A popup document fitted to its content: the floating card lies in flow at the origin and
    /// no outside catcher covers the surface. Written `data-extent="popup"`.
    Popup,
}

impl RootExtent {
    /// The `data-extent` value, written only for a viewport or popup root, so a content root's
    /// markup is what it was.
    pub fn attribute(self) -> Option<&'static str> {
        match self {
            RootExtent::Content => None,
            RootExtent::Viewport => Some("viewport"),
            RootExtent::Popup => Some("popup"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RootExtent;

    #[test]
    fn only_a_viewport_or_popup_root_writes_its_extent() {
        assert_eq!(RootExtent::default().attribute(), None);
        assert_eq!(RootExtent::Viewport.attribute(), Some("viewport"));
        assert_eq!(RootExtent::Popup.attribute(), Some("popup"));
    }
}
