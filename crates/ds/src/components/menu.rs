//! Menu: "ONE MENU: every list of choices in the window uses this" (design/04-COMPONENTS.md
//! section 20). Filtering and ranking are pure Rust (design/06-INTERACTIONS.md section 11).

use crate::components::menu_entry::MenuEntry;
use crate::geometry::Anchor;
use dioxus::prelude::*;

/// Which menu shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuKind {
    /// 280 wide, 34 px tiles, title and help and shortcut.
    Rich,
    /// 220 wide, 22 px tiles.
    Slim,
    /// C's check-column menu, anchored under its button's right edge.
    Dropdown,
    /// Anchored at the pointer.
    Context,
}

/// Whether typing filters the entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Filter {
    /// Typing filters with the fuzzy ranker and resets the selection.
    Typing,
    /// The entries are fixed.
    #[default]
    None,
}

/// A floating list of choices.
#[component]
pub fn Menu<T: Clone + PartialEq + 'static>(
    kind: MenuKind,
    anchor: Anchor,
    entries: Vec<MenuEntry<T>>,
    #[props(default)] filter: Filter,
    onpick: EventHandler<T>,
    onclose: EventHandler<()>,
) -> Element {
    todo!()
}
