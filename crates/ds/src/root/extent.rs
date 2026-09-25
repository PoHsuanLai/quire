//! How tall and wide a root is: as its content (a window, the bar, a card) or the whole viewport
//! (an overlay surface), sill FINDINGS Q94.
//!
//! A `.ds` root is a block in the document's flow, so it is as tall as what it lays out. A root
//! whose content is all positioned (a centred `Sheet`, an OSD card, a click catcher) lays out
//! nothing and is 0 px tall; on Blitz so is everything above it (dioxus-native-dom's `#main` is
//! `height:auto`), and a `position:fixed; inset:0` child does not help, since Taffy places a
//! fixed box against its parent rather than the viewport (sill F172). A `Viewport` root claims
//! the viewport's size itself, with `min-height:100vh; min-width:100vw`, so what it positions has
//! the surface's whole area to be placed in.

/// How big a root is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RootExtent {
    /// As big as its content, the document's usual rule: a window, the bar, a card.
    #[default]
    Content,
    /// At least the viewport: an overlay surface whose content is positioned (a sheet, an OSD,
    /// a catcher). Written `data-extent="viewport"`.
    Viewport,
}

impl RootExtent {
    /// The `data-extent` value, written only for a viewport root, so a content root's markup is
    /// what it was.
    pub(crate) fn attribute(self) -> Option<&'static str> {
        match self {
            RootExtent::Content => None,
            RootExtent::Viewport => Some("viewport"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RootExtent;

    #[test]
    fn only_a_viewport_root_writes_its_extent() {
        assert_eq!(RootExtent::default().attribute(), None);
        assert_eq!(RootExtent::Viewport.attribute(), Some("viewport"));
    }
}
