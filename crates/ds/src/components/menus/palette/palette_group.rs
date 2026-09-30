//! A command palette's groups: a titled run of rows or an emoji grid, with an optional action on
//! its header ("Show More"). `CommandPalette { groups }` takes [`PaletteGroups`], which a `Vec`
//! of groups converts into.

use crate::components::content::text_runs::TextLine;
use crate::components::lists::emoji_grid::grid::EmojiCells;
use crate::components::lists::row::accessory::Accessory;
use crate::components::lists::row::action::RowAction;
use crate::components::lists::row::chord::RowChord;
use crate::components::lists::row::leading::RowLeading;
use crate::components::lists::row::shape::RowShape;
use dioxus::prelude::*;
use ds_core::vocab::Availability;

/// A result the palette lists: what picking it yields and how its `Row` draws.
#[derive(Debug, Clone, PartialEq)]
pub struct PaletteRow<T> {
    /// What picking it yields.
    pub value: T,
    /// Its name. A plain title is marked where the query matches it; runs are drawn as given (the
    /// caller's marks win).
    pub title: TextLine,
    /// One line of help.
    pub detail: Option<TextLine>,
    /// What leads it.
    pub leading: RowLeading,
    /// What ends it.
    pub accessory: Accessory,
    /// The keys of its action, after the accessory: by default only while it is the selection
    /// (Spotlight's hint, [`RowChord::on_selected`]); none when empty.
    pub chord: RowChord,
    /// Whether it can be picked.
    pub availability: Availability,
    /// A button at its end that acts without picking it.
    pub action: Option<RowAction>,
    /// How it draws beyond its title and detail: `Plain`, or a file's or a clipboard entry's
    /// shape.
    pub shape: RowShape,
}

impl<T> PaletteRow<T> {
    /// An enabled row yielding `value`, named `title`, with nothing else. Set what differs:
    /// `PaletteRow { accessory, ..PaletteRow::new(hit, title) }`.
    pub fn new(value: T, title: impl Into<TextLine>) -> Self {
        PaletteRow {
            value,
            title: title.into(),
            detail: None,
            leading: RowLeading::None,
            accessory: Accessory::None,
            chord: RowChord::default(),
            availability: Availability::Enabled,
            action: None,
            shape: RowShape::Plain,
        }
    }
}

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
    /// Rows.
    List(Vec<PaletteRow<T>>),
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
    pub fn list(title: impl Into<String>, entries: Vec<PaletteRow<T>>) -> Self {
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

/// A palette's groups, in order: what `CommandPalette { groups }` takes, built from a
/// `Vec<PaletteGroup<T>>`.
#[derive(Debug, Clone, PartialEq)]
pub struct PaletteGroups<T: 'static>(pub Vec<PaletteGroup<T>>);

/// The order a Space wants its result groups in (design/30 section 2.11): the launcher's results
/// come grouped by kind, each under its `SectionHeader`, and each Space lists the kinds it cares
/// about first. `G` is the caller's own kind (`Apps`, `Files`, `Commands`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GroupOrder<G>(pub Vec<G>);

impl<G: PartialEq> GroupOrder<G> {
    /// `groups`, each tagged with its kind, in this order: the kinds named here first, in the
    /// order named, then any others in the order given. Groups of one kind keep their order.
    pub fn arrange<T>(&self, groups: Vec<(G, PaletteGroup<T>)>) -> PaletteGroups<T> {
        let rank = |kind: &G| self.0.iter().position(|named| named == kind);
        let mut tagged: Vec<(usize, (G, PaletteGroup<T>))> =
            groups.into_iter().enumerate().collect();
        tagged.sort_by_key(|(given, (kind, _))| match rank(kind) {
            Some(place) => (0, place, *given),
            None => (1, 0, *given),
        });
        PaletteGroups(tagged.into_iter().map(|(_, (_, group))| group).collect())
    }
}

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

/// No groups: the palette shows its `empty` line.
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

#[cfg(test)]
mod tests {
    use super::{GroupOrder, PaletteGroup};

    fn titles(order: &GroupOrder<char>, kinds: &[(char, &str)]) -> Vec<String> {
        let groups = kinds
            .iter()
            .map(|(kind, title)| (*kind, PaletteGroup::<u8>::list(*title, Vec::new())))
            .collect();
        order
            .arrange(groups)
            .0
            .into_iter()
            .map(|group| group.title)
            .collect()
    }

    #[test]
    fn a_space_lists_its_kinds_first_and_the_rest_as_given() {
        let given = [
            ('a', "Apps"),
            ('f', "Files"),
            ('c', "Commands"),
            ('e', "Emoji"),
        ];
        // (the Space's order, the titles that result)
        let cases: &[(&[char], &[&str])] = &[
            (&[], &["Apps", "Files", "Commands", "Emoji"]),
            (&['c', 'a'], &["Commands", "Apps", "Files", "Emoji"]),
            (&['e'], &["Emoji", "Apps", "Files", "Commands"]),
            (&['x', 'f'], &["Files", "Apps", "Commands", "Emoji"]),
        ];
        for (order, want) in cases {
            assert_eq!(
                titles(&GroupOrder(order.to_vec()), &given),
                *want,
                "{order:?}"
            );
        }
    }
}
