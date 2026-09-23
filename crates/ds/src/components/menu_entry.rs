//! MenuEntry: what a Menu or CommandPalette lists (design/04-COMPONENTS.md section 20).
//! Data only: the menu renders it.

use crate::components::avatar::AvatarFace;
use crate::components::vocab::{Check, Shortcut};
use crate::icon::Icon;

/// The tile at an item's start.
#[derive(Debug, Clone, PartialEq)]
pub enum Tile {
    /// A glyph.
    Icon(Icon),
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
    },
    /// A group title.
    Header(String),
    /// A rule between groups.
    Separator,
}
