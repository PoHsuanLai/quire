//! MenuItem: what a `Menu` lists, as data (`NSMenuItem`, design/30 section 2.4): a command with
//! its image, key equivalent and check state, a parent of a submenu, a section header, a status
//! line and a separator. `view` draws one item, `lines` the whole panel.

use crate::components::content::avatar::AvatarFace;
use crate::components::content::icon_source::IconSource;
use crate::components::menus::item::text::{ItemText, Marks};
use ds_core::vocab::{Availability, Check, Shortcut};
use ds_style::icon::Icon;
use ds_style::tokens::system_hue::SystemHue;

/// The image at an item's start.
#[derive(Debug, Clone, PartialEq)]
pub enum MenuImage {
    /// A glyph.
    Icon(Icon),
    /// Any icon source: an app's icon file (drawn as it is, with no plate under it), a symbolic
    /// icon (in the text colour) or a glyph.
    Source(IconSource),
    /// A person or account as a coloured disc with a letter: a result row for someone.
    Avatar(AvatarFace),
    /// A coloured dot in a tag's colour (`--tag-*`): a row for a Finder tag, in the menu that
    /// tags files. It keeps its colour under the highlight.
    Swatch(SystemHue),
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
        /// arrow keys and ignores the pointer; a context menu leaves it out unless it is asked
        /// to dim it (`ContextUnavailable`; design/13 section 13.3.3).
        availability: Availability,
        /// Whether a pick closes the menu.
        after: AfterPick,
        /// What the row says beside its title: the characters a query matched and a second line.
        text: ItemText,
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
    /// One row of coloured dots, each its own pick target: Finder's row of tag colours in a
    /// context menu. Left and Right move across the dots when the row is highlighted; Up and
    /// Down pass the row as one stop.
    Swatches(SwatchRow<T>),
    /// A rule between groups.
    Separator,
}

/// A [`MenuItem::Swatches`] row: the dots, and whether they can be picked.
#[derive(Debug, Clone, PartialEq)]
pub struct SwatchRow<T> {
    /// What the row is called to assistive technology (the group's label); nothing is drawn.
    pub title: String,
    /// Whether the dots can be picked. A disabled row is drawn at .35 opacity and skipped by the
    /// arrow keys, as a disabled item is.
    pub availability: Availability,
    /// The dots, in order.
    pub swatches: Vec<MenuSwatch<T>>,
}

impl<T> SwatchRow<T> {
    fn map<U>(self, lift: &impl Fn(T) -> U) -> SwatchRow<U> {
        SwatchRow {
            title: self.title,
            availability: self.availability,
            swatches: self
                .swatches
                .into_iter()
                .map(|swatch| MenuSwatch {
                    value: lift(swatch.value),
                    name: swatch.name,
                    colour: swatch.colour,
                    check: swatch.check,
                    after: swatch.after,
                })
                .collect(),
        }
    }
}

/// One dot of a [`SwatchRow`].
#[derive(Debug, Clone, PartialEq)]
pub struct MenuSwatch<T> {
    /// What picking it yields.
    pub value: T,
    /// Its name: the tooltip's text and the dot's `aria-label`.
    pub name: String,
    /// Its colour (`--tag-*`).
    pub colour: SystemHue,
    /// Whether the tag is already on the files: `On` draws a check in the dot, `Mixed` a dash.
    pub check: Option<Check>,
    /// Whether a pick closes the menu.
    pub after: AfterPick,
}

impl<T> MenuSwatch<T> {
    /// A dot named `name` in `colour` that yields `value`, shows no state and closes the menu.
    pub fn new(value: T, name: impl Into<String>, colour: SystemHue) -> Self {
        MenuSwatch {
            value,
            name: name.into(),
            colour,
            check: None,
            after: AfterPick::Close,
        }
    }

    /// The same dot with the state mark `check`.
    pub fn with_check(self, check: Check) -> Self {
        MenuSwatch {
            check: Some(check),
            ..self
        }
    }

    /// The same dot with `after` as what its pick does to the menu.
    pub fn with_after(self, after: AfterPick) -> Self {
        MenuSwatch { after, ..self }
    }
}

impl<T> MenuItem<T> {
    /// The same item yielding `lift` of its value, in every submenu too: an app's rows inside a
    /// menu of a wider pick type.
    pub fn map<U>(self, lift: &impl Fn(T) -> U) -> MenuItem<U> {
        match self {
            MenuItem::Item {
                value,
                title,
                image,
                key,
                hint,
                check,
                availability,
                after,
                text,
            } => MenuItem::Item {
                value: lift(value),
                title,
                image,
                key,
                hint,
                check,
                availability,
                after,
                text,
            },
            MenuItem::Submenu {
                title,
                image,
                availability,
                children,
            } => MenuItem::Submenu {
                title,
                image,
                availability,
                children: children.into_iter().map(|child| child.map(lift)).collect(),
            },
            MenuItem::Header(title) => MenuItem::Header(title),
            MenuItem::Swatches(row) => MenuItem::Swatches(row.map(lift)),
            MenuItem::Info { title, detail } => MenuItem::Info { title, detail },
            MenuItem::Separator => MenuItem::Separator,
        }
    }

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
            text: ItemText::default(),
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

    /// A row of `swatches`, called `title` to assistive technology, enabled.
    pub fn swatches(title: impl Into<String>, swatches: Vec<MenuSwatch<T>>) -> Self {
        MenuItem::Swatches(SwatchRow {
            title: title.into(),
            availability: Availability::Enabled,
            swatches,
        })
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
        match &mut self {
            MenuItem::Item { availability, .. } | MenuItem::Submenu { availability, .. } => {
                *availability = to;
            }
            MenuItem::Swatches(row) => row.availability = to,
            MenuItem::Header(_) | MenuItem::Info { .. } | MenuItem::Separator => {}
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

    /// The same item with the characters of its title that a query matched drawn emphasised; only
    /// a command has a title to mark.
    pub fn with_marks(mut self, marks: Marks) -> Self {
        if let MenuItem::Item { text, .. } = &mut self {
            text.marks = marks;
        }
        self
    }

    /// The same item with a tooltip `tip`, shown on hover even while the item is disabled (the
    /// reason it is); only a command has one.
    pub fn with_tip(mut self, tip: impl Into<String>) -> Self {
        if let MenuItem::Item { text, .. } = &mut self {
            text.tip = Some(tip.into());
        }
        self
    }

    /// The same item with a second, fainter line under its title; only a command has one.
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        if let MenuItem::Item { text, .. } = &mut self {
            text.subtitle = Some(subtitle.into());
        }
        self
    }

    /// Whether the keys may rest on it.
    pub(crate) fn availability(&self) -> Option<Availability> {
        match self {
            MenuItem::Item { availability, .. } | MenuItem::Submenu { availability, .. } => {
                Some(*availability)
            }
            MenuItem::Swatches(row) => Some(row.availability),
            MenuItem::Header(_) | MenuItem::Info { .. } | MenuItem::Separator => None,
        }
    }
}
