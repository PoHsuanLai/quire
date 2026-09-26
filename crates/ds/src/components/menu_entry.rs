//! MenuEntry: what a Menu or CommandPalette lists (design/04-COMPONENTS.md section 20), as
//! data; `menu_item` draws one item and `menu_match` holds the fuzzy matcher that marks a title
//! (design/06-INTERACTIONS.md section 11.2). Disabled items and submenus follow
//! design/13-BEHAVIOUR-menus-windows.md sections 13.3.3 and 13.3.4.

use crate::components::avatar::AvatarFace;
use crate::components::row_action::RowAction;
use crate::components::row_shape::RowShape;
use crate::components::text_runs::Text;
use crate::components::vocab::{Availability, Check, Shortcut};
use crate::icon::{Icon, IconSource};

pub(crate) use crate::components::menu_match::fuzzy;

/// The tile at an item's start.
#[derive(Debug, Clone, PartialEq)]
pub enum Tile {
    /// A glyph.
    Icon(Icon),
    /// Any icon source: an app's icon file (drawn as it is, filling the tile, with no plate of
    /// its own under it), a symbolic icon (on the plate, in the text colour) or a glyph (sill
    /// FINDINGS Q42).
    Source(IconSource),
    /// A letter or two.
    Text(String),
    /// An avatar.
    Avatar(AvatarFace),
}

/// What trails an item.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Trail {
    /// Nothing.
    #[default]
    None,
    /// A shortcut, as glyphs.
    Shortcut(Shortcut),
    /// A short note.
    Note(String),
}

/// One line of a menu.
#[derive(Debug, Clone, PartialEq)]
pub enum MenuEntry<T> {
    /// A choice.
    Item {
        /// What picking it yields.
        value: T,
        /// Its name.
        title: String,
        /// One line of help.
        detail: Option<String>,
        /// Its tile.
        tile: Option<Tile>,
        /// Its trail.
        trail: Trail,
        /// Its check mark, for a menu of toggles.
        check: Option<Check>,
        /// Whether it can be picked. A disabled item is drawn at .35 opacity, skipped by the
        /// arrow keys and ignores the pointer (design/13 section 13.3.3).
        availability: Availability,
    },
    /// An item that opens a submenu of `children` beside it (design/13 section 13.3.4): on a
    /// 200 ms rest, or at once on Right, Enter or a click. It shows a chevron where an item
    /// shows its trail, and picks nothing itself.
    Submenu {
        /// Its name.
        title: String,
        /// Its tile.
        tile: Option<Tile>,
        /// Whether it opens. A disabled one is drawn and skipped as a disabled item is.
        availability: Availability,
        /// What the submenu lists; a picked child's value reaches the menu's `onpick`.
        children: Vec<MenuEntry<T>>,
    },
    /// A group title.
    Header(String),
    /// A status line (a network's address, a battery's time left): drawn at an item's weight
    /// without the header's eyebrow, never a choice, so the keys and the pointer pass it by.
    Info {
        /// The line.
        title: String,
        /// A second, fainter line.
        detail: Option<String>,
    },
    /// A rule between groups.
    Separator,
    /// A choice whose title and detail are [`Text`] runs the caller computed (a search's
    /// marks, a name stronger than its path) and which may end in its own action (mailo gaps
    /// 2). Picked, navigated and filtered exactly as an [`MenuEntry::Item`].
    Row(MenuRow<T>),
}

/// A choice with runs in its words and an optional trailing action: what
/// [`MenuEntry::Row`] holds. Build one with [`MenuRow::new`] and set what differs:
/// `MenuRow { trailing: Some(remove), ..MenuRow::new(hit, title) }`.
#[derive(Debug, Clone, PartialEq)]
pub struct MenuRow<T> {
    /// What picking it yields.
    pub value: T,
    /// Its name. A plain title is marked where a palette's query or a menu's filter matches
    /// it; runs are drawn as given (the caller's marks win).
    pub title: Text,
    /// One line of help.
    pub detail: Option<Text>,
    /// Its tile.
    pub tile: Option<Tile>,
    /// Its trail.
    pub trail: Trail,
    /// Its check mark, for a menu of toggles.
    pub check: Option<Check>,
    /// Whether it can be picked.
    pub availability: Availability,
    /// A button at its end that acts without picking it.
    pub trailing: Option<RowAction>,
    /// How it draws beyond its title and detail: `Plain`, or a file's or a clipboard entry's
    /// shape (sill Q290). Picked, navigated and matched on its title whatever its shape.
    pub shape: RowShape,
}

impl<T> MenuRow<T> {
    /// An enabled row yielding `value`, named `title`, with nothing else.
    pub fn new(value: T, title: impl Into<Text>) -> Self {
        MenuRow {
            value,
            title: title.into(),
            detail: None,
            tile: None,
            trail: Trail::None,
            check: None,
            availability: Availability::Enabled,
            trailing: None,
            shape: RowShape::Plain,
        }
    }
}

impl<T> From<MenuRow<T>> for MenuEntry<T> {
    fn from(row: MenuRow<T>) -> Self {
        MenuEntry::Row(row)
    }
}

impl<T> MenuEntry<T> {
    /// A choice's title as plain characters, for the fuzzy matcher; `None` for a header, a
    /// status line or a rule.
    pub(crate) fn match_title(&self) -> Option<String> {
        match self {
            MenuEntry::Item { title, .. } | MenuEntry::Submenu { title, .. } => Some(title.clone()),
            MenuEntry::Row(row) => Some(row.title.plain_text()),
            MenuEntry::Header(_) | MenuEntry::Info { .. } | MenuEntry::Separator => None,
        }
    }

    /// Whether the query's marks may be drawn over the title: not over a row's runs.
    pub(crate) fn takes_marks(&self) -> bool {
        match self {
            MenuEntry::Item { .. } | MenuEntry::Submenu { .. } => true,
            MenuEntry::Row(row) => matches!(row.title, Text::Plain(_)),
            MenuEntry::Header(_) | MenuEntry::Info { .. } | MenuEntry::Separator => false,
        }
    }
}
