//! Where a menu is and how it differs from the others (design/30 section 2.4): the three
//! placements, where each goes against its anchor and what each shows. One density for all
//! (row 22, separator 9, inset 5, radius 8); every menu opens at once.

use ds_core::geometry::{
    placement::{Align, Placement, Side},
    units::{Px, Rect},
};
use ds_core::word::Word;

/// The panel's padding: rows sit this far inside its edge, and a submenu's top sits this far
/// above its parent row.
pub(crate) const MENU_INSET: Px = Px(5.0);

/// Where a menu belongs (`NSMenu`'s three roles).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum MenuPlacement {
    /// A menu bar menu, hung under its title. Its key equivalents show.
    Bar,
    /// A pop-up or pull-down button's menu, a Dock or status-item menu: below its anchor.
    #[default]
    Popup,
    /// A context menu, at the pointer: it hides what is unavailable and shows no key
    /// equivalents (design/27).
    Context,
}

/// Whether a menu draws its items' key equivalents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Keys {
    Shown,
    Hidden,
}

impl MenuPlacement {
    /// Where the menu goes against its anchor: a bar menu flush under its title, a pop-up two
    /// below its button, a context menu at the pointer.
    pub(crate) fn placement(self, anchor: Rect) -> (Rect, Placement, Px) {
        let gap = match self {
            MenuPlacement::Bar | MenuPlacement::Context => Px(0.0),
            MenuPlacement::Popup => Px(2.0),
        };
        (anchor, Placement::new(Side::Bottom, Align::Start), gap)
    }

    /// Whether key equivalents are drawn.
    pub(crate) fn keys(self) -> Keys {
        match self {
            MenuPlacement::Bar | MenuPlacement::Popup => Keys::Shown,
            MenuPlacement::Context => Keys::Hidden,
        }
    }
}
