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

/// What a pick does to the menu that holds the item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AfterPick {
    /// The item blinks twice, the menu yields the value and fades out: `NSMenu`'s way.
    #[default]
    Close,
    /// The value is yielded at once and the menu stays up, so the person can pick again: a
    /// toggle in a set of toggles (a label to add or remove), as SwiftUI's
    /// `menuActionDismissBehavior(.disabled)` does on the Mac. Nothing blinks.
    KeepOpen,
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
        /// A short faint word at the item's end, before the key equivalent: the time a snooze
        /// lands on, what a trigger types. The Mac's menu has no second line; what it adds to a
        /// title goes at the trailing end (design/30 section 2.4). Unlike a key equivalent, a
        /// context menu shows it.
        hint: Option<String>,
        /// Its state mark, for an item that is on, off or mixed: a check, a dash for `Mixed`.
        check: Option<Check>,
        /// Whether it can be picked. A disabled item is drawn at .35 opacity, skipped by the
        /// arrow keys and ignores the pointer; a context menu leaves it out (design/13 section
        /// 13.3.3).
        availability: Availability,
        /// Whether a pick closes the menu.
        after: AfterPick,
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
            hint: None,
            check: None,
            availability: Availability::Enabled,
            after: AfterPick::Close,
        }
    }

    /// The same item with `image`; a header, status line or rule has none to change.
    pub fn with_image(mut self, image: MenuImage) -> Self {
        if let MenuItem::Item { image: slot, .. } | MenuItem::Submenu { image: slot, .. } =
            &mut self
        {
            *slot = Some(image);
        }
        self
    }

    /// The same item with the key equivalent `key`; only a command has one.
    pub fn with_key(mut self, key: Shortcut) -> Self {
        if let MenuItem::Item { key: slot, .. } = &mut self {
            *slot = Some(key);
        }
        self
    }

    /// The same item with the trailing `hint`; only a command has one.
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        if let MenuItem::Item { hint: slot, .. } = &mut self {
            *slot = Some(hint.into());
        }
        self
    }

    /// The same item with the state mark `check`; only a command has one.
    pub fn with_check(mut self, check: Check) -> Self {
        if let MenuItem::Item { check: slot, .. } = &mut self {
            *slot = Some(check);
        }
        self
    }

    /// The same item with `availability`; a header, status line or rule has none to change.
    pub fn with_availability(mut self, to: Availability) -> Self {
        if let MenuItem::Item { availability, .. } | MenuItem::Submenu { availability, .. } =
            &mut self
        {
            *availability = to;
        }
        self
    }

    /// The same item with `after` as what its pick does to the menu; only a command has one.
    pub fn with_after(mut self, to: AfterPick) -> Self {
        if let MenuItem::Item { after, .. } = &mut self {
            *after = to;
        }
        self
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
