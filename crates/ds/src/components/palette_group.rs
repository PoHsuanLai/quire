//! A command palette's groups (sill Q294): a titled run of rows or an emoji grid, with an
//! optional action on its header ("Show More"). `CommandPalette { groups }` takes
//! [`PaletteGroups`], which a `Vec` of groups converts into, and so does the older
//! `Vec<(String, Vec<MenuEntry<T>>)>`: every call site written before groups had actions still
//! compiles as it was.

use crate::components::emoji_grid::EmojiCells;
use crate::components::menu_entry::MenuEntry;
use dioxus::prelude::*;

/// One group: its title (a `SectionHeader`), what it lists, and the header's trailing action.
#[derive(Clone, PartialEq)]
pub struct PaletteGroup<T: 'static> {
    /// The header's words.
    pub title: String,
    /// Its rows or its grid. A group with none is not drawn, header and action included.
    pub entries: GroupEntries<T>,
    /// The header's trailing action, its words and what it does ("Show More"): drawn as
    /// `SectionHeader`'s action, and a stop of the palette's cursor after the group's last row
    /// or cell, so Down reaches it and Enter runs it (the palette stays open).
    pub action: Option<(String, EventHandler<()>)>,
}

/// What a group lists.
#[derive(Debug, Clone, PartialEq)]
pub enum GroupEntries<T> {
    /// Rows, as a menu's entries.
    List(Vec<MenuEntry<T>>),
    /// An emoji grid, moved through in two dimensions.
    Grid(EmojiCells<T>),
}

impl<T> GroupEntries<T> {
    /// Whether there is nothing to draw.
    pub(crate) fn is_empty(&self) -> bool {
        match self {
            GroupEntries::List(entries) => entries.is_empty(),
            GroupEntries::Grid(grid) => grid.cells.is_empty(),
        }
    }
}

impl<T> PaletteGroup<T> {
    /// A group of rows with no action.
    pub fn list(title: impl Into<String>, entries: Vec<MenuEntry<T>>) -> Self {
        PaletteGroup {
            title: title.into(),
            entries: GroupEntries::List(entries),
            action: None,
        }
    }

    /// A group holding an emoji grid, with no action.
    pub fn grid(title: impl Into<String>, grid: EmojiCells<T>) -> Self {
        PaletteGroup {
            title: title.into(),
            entries: GroupEntries::Grid(grid),
            action: None,
        }
    }

    /// The same group with `label` as its header's action, running `run`.
    pub fn with_action(self, label: impl Into<String>, run: EventHandler<()>) -> Self {
        PaletteGroup {
            action: Some((label.into(), run)),
            ..self
        }
    }
}

impl<T> From<(String, Vec<MenuEntry<T>>)> for PaletteGroup<T> {
    fn from((title, entries): (String, Vec<MenuEntry<T>>)) -> Self {
        PaletteGroup::list(title, entries)
    }
}

/// A palette's groups, in order: what `CommandPalette { groups }` takes. Built from a
/// `Vec<PaletteGroup<T>>` or, as before groups had actions and grids, from a
/// `Vec<(String, Vec<MenuEntry<T>>)>`.
#[derive(Debug, Clone, PartialEq)]
pub struct PaletteGroups<T: 'static>(pub Vec<PaletteGroup<T>>);

/// What the groups draw, without the actions' handlers (a caller's render makes new ones each
/// time, and a new handler is not a new result): what the palette's rect book compares.
pub(crate) type GroupsKey<T> = Vec<(String, GroupEntries<T>, Option<String>)>;

impl<T: Clone> PaletteGroups<T> {
    /// The groups' titles, entries and action labels.
    pub(crate) fn key(&self) -> GroupsKey<T> {
        self.0
            .iter()
            .map(|group| {
                (
                    group.title.clone(),
                    group.entries.clone(),
                    group.action.as_ref().map(|(label, _)| label.clone()),
                )
            })
            .collect()
    }
}

/// No groups: the palette shows its `empty` line. (`groups: Vec::new()` no longer names its
/// element type now that two kinds of `Vec` convert; write `PaletteGroups::default()`.)
impl<T> Default for PaletteGroups<T> {
    fn default() -> Self {
        PaletteGroups(Vec::new())
    }
}

impl<T> From<Vec<PaletteGroup<T>>> for PaletteGroups<T> {
    fn from(groups: Vec<PaletteGroup<T>>) -> Self {
        PaletteGroups(groups)
    }
}

impl<T> From<Vec<(String, Vec<MenuEntry<T>>)>> for PaletteGroups<T> {
    fn from(groups: Vec<(String, Vec<MenuEntry<T>>)>) -> Self {
        PaletteGroups(groups.into_iter().map(PaletteGroup::from).collect())
    }
}

impl<T> std::fmt::Debug for PaletteGroup<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PaletteGroup")
            .field("title", &self.title)
            .field("entries", &self.entries)
            .field(
                "action",
                &self.action.as_ref().map(|(label, _)| label.as_str()),
            )
            .finish()
    }
}
