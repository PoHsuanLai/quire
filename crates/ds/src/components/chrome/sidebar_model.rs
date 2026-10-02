//! What a `Sidebar` is made of: its sections, top to bottom, and what it is drawn on.

use crate::components::lists::list::model::ListItem;
use dioxus::prelude::*;
use ds_core::word::Word;

/// One section of a sidebar.
#[derive(Debug, Clone, PartialEq)]
pub enum SidebarSection<K> {
    /// A source-list `List`. Its group headings are its own `ListItem::heading`s, and its rows
    /// share the sidebar's one cursor.
    List(Vec<ListItem<K>>),
    /// Anything else a sidebar holds: pinned tiles, Today tabs. The caller draws its heading
    /// (a `SectionHeader`) and its selection, and a section that is empty is simply left out.
    Custom(Element),
}

/// What a sidebar paints behind its rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum SidebarFill {
    /// The sidebar's own ground, the source list as a Mac window draws it.
    #[default]
    Material,
    /// Nothing: whatever the window draws behind the sidebar shows, such as a Space's flat tint.
    /// The inks are the ordinary ones.
    Clear,
}
