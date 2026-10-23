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
    /// Anything that takes the height the other sections leave, and is given that height as a
    /// definite one: a virtualised list (PDF thumbnails, an outline) under a few view rows, as
    /// Preview's sidebar is. The sections above keep their natural height and the body no longer
    /// scrolls as a whole; the content scrolls itself. A sidebar holds one: a second `Fill` is a
    /// mistake (a debug assertion fails) and is left out.
    Fill(Element),
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
    /// A flat ground: `--surface-2`, the first rung above `--paper` on the lightness ladder, in
    /// one tone (no gradient, no Space hue). For content-first apps (a terminal, an editor) whose
    /// content sits on `--paper`: the sidebar reads as the same product, one step lifted. The
    /// inks are the ordinary ones.
    Flat,
}
