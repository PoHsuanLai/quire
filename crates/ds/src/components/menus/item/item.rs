//! MenuItem: what a `Menu` lists, as data (`NSMenuItem`, design/30 section 2.4): a command with
//! its image, key equivalent and check state, a parent of a submenu, a section header, a status
//! line and a separator. `view` draws one item, `lines` the whole panel.

use crate::components::content::icon_source::IconSource;
use ds_core::vocab::{Availability, Check, Shortcut};
use ds_style::icon::Icon;

/// The image at an item's start.
#[derive(Debug, Clone, PartialEq)]
pub enum MenuImage {
    /// A glyph.
    Icon(Icon),
    /// Any icon source: an app's icon file (drawn as it is, with no plate under it), a symbolic
    /// icon (in the text colour) or a glyph.
    Source(IconSource),
}

/// One line of a menu.
#[derive(Debug, Clone, PartialEq)]
pub enum MenuItem<T> {
    /// A command.
    Item {
        /// What picking it yields.
        value: T,
        /// Its name.
        title: String,
        /// Its image.
        image: Option<MenuImage>,
        /// Its key equivalent, drawn as glyphs at the item's end. A context menu shows none.
        key: Option<Shortcut>,
        /// Its state mark, for an item that is on, off or mixed: a check, a dash for `Mixed`.
        check: Option<Check>,
        /// Whether it can be picked. A disabled item is drawn at .35 opacity, skipped by the
        /// arrow keys and ignores the pointer; a context menu leaves it out (design/13 section
        /// 13.3.3).
        availability: Availability,
    },
    /// An item that opens a submenu of `children` beside it (design/13 section 13.3.4): on a
    /// 200 ms rest, or at once on Right, Enter or a click. It shows a chevron where an item
    /// shows its key equivalent, and picks nothing itself.
    Submenu {
        /// Its name.
        title: String,
        /// Its image.
        image: Option<MenuImage>,
        /// Whether it opens. A disabled one is drawn and skipped as a disabled item is.
        availability: Availability,
        /// What the submenu lists; a picked child's value reaches the menu's `onpick`.
        children: Vec<MenuItem<T>>,
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
}

impl<T> MenuItem<T> {
    /// An enabled command yielding `value`, named `title`, with nothing else.
    pub fn new(value: T, title: impl Into<String>) -> Self {
        MenuItem::Item {
            value,
            title: title.into(),
            image: None,
            key: None,
            check: None,
            availability: Availability::Enabled,
        }
    }

    /// The same item with `image`; a header, status line or rule has none to change.
    pub fn with_image(self, image: MenuImage) -> Self {
        match self {
            MenuItem::Item {
                value,
                title,
                key,
                check,
                availability,
                ..
            } => MenuItem::Item {
                value,
                title,
                image: Some(image),
                key,
                check,
                availability,
            },
            MenuItem::Submenu {
                title,
                availability,
                children,
                ..
            } => MenuItem::Submenu {
                title,
                image: Some(image),
                availability,
                children,
            },
            other @ (MenuItem::Header(_) | MenuItem::Info { .. } | MenuItem::Separator) => other,
        }
    }

    /// The same item with the key equivalent `key`; only a command has one.
    pub fn with_key(self, key: Shortcut) -> Self {
        match self {
            MenuItem::Item {
                value,
                title,
                image,
                check,
                availability,
                ..
            } => MenuItem::Item {
                value,
                title,
                image,
                key: Some(key),
                check,
                availability,
            },
            other @ (MenuItem::Submenu { .. }
            | MenuItem::Header(_)
            | MenuItem::Info { .. }
            | MenuItem::Separator) => other,
        }
    }

    /// The same item with the state mark `check`; only a command has one.
    pub fn with_check(self, check: Check) -> Self {
        match self {
            MenuItem::Item {
                value,
                title,
                image,
                key,
                availability,
                ..
            } => MenuItem::Item {
                value,
                title,
                image,
                key,
                check: Some(check),
                availability,
            },
            other @ (MenuItem::Submenu { .. }
            | MenuItem::Header(_)
            | MenuItem::Info { .. }
            | MenuItem::Separator) => other,
        }
    }

    /// The same item with `availability`; a header, status line or rule has none to change.
    pub fn with_availability(self, to: Availability) -> Self {
        match self {
            MenuItem::Item {
                value,
                title,
                image,
                key,
                check,
                ..
            } => MenuItem::Item {
                value,
                title,
                image,
                key,
                check,
                availability: to,
            },
            MenuItem::Submenu {
                title,
                image,
                children,
                ..
            } => MenuItem::Submenu {
                title,
                image,
                availability: to,
                children,
            },
            other @ (MenuItem::Header(_) | MenuItem::Info { .. } | MenuItem::Separator) => other,
        }
    }

    /// Whether the keys may rest on it.
    pub(crate) fn availability(&self) -> Option<Availability> {
        match self {
            MenuItem::Item { availability, .. } | MenuItem::Submenu { availability, .. } => {
                Some(*availability)
            }
            MenuItem::Header(_) | MenuItem::Info { .. } | MenuItem::Separator => None,
        }
    }
}
