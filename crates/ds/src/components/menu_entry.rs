//! MenuEntry: what a Menu or CommandPalette lists (design/04-COMPONENTS.md section 20).
//! The data, and how one item draws (shared by `Menu` and `CommandPalette`), with the fuzzy
//! matcher that marks a title (design/06-INTERACTIONS.md section 11.2). Disabled items and
//! submenus follow design/13-BEHAVIOUR-menus-windows.md sections 13.3.3 and 13.3.4.

use crate::components::avatar::{AvatarFace, face};
use crate::components::icon_view::IconView;
use crate::components::press::{PointerButton, Press, button_of};
use crate::components::vocab::{Availability, Check, Selection, Shortcut, Switch};
use crate::geometry::{Point, Px};
use crate::icon::render::{Glyph, IconSize};
use crate::icon::{Icon, IconSource};
use dioxus::prelude::*;

pub(crate) use crate::components::menu_match::{fuzzy, marked};

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
}

/// Which column layout an item row takes: the Rich/Slim/Context grid with a tile, or the
/// Dropdown's check column (design/04-COMPONENTS.md section 20).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Row {
    /// A tile, the title and detail, a trail.
    Tiled,
    /// A check column, the title, a trail.
    Checked,
}

/// Whether a row opens a submenu, and whether that submenu is open (`aria-expanded`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Branch {
    /// It picks a value.
    Leaf,
    /// It opens a submenu, open or not.
    Parent(Switch),
}

/// One item as a menu draws it: its words, which characters of the title matched the query,
/// and whether it is the keyboard selection.
pub(crate) struct ItemView<'a> {
    /// The title.
    pub title: &'a str,
    /// One line of help.
    pub detail: Option<&'a str>,
    /// The tile.
    pub tile: Option<&'a Tile>,
    /// The trail.
    pub trail: &'a Trail,
    /// The check mark.
    pub check: Option<Check>,
    /// The title's matched characters, by char index.
    pub marks: &'a [usize],
    /// Whether this is the selected item.
    pub selection: Selection,
    /// Whether it can be picked.
    pub availability: Availability,
    /// Whether it opens a submenu.
    pub branch: Branch,
}

/// What a row reports: a click of a live row (`pick`; a parent opens its submenu), the
/// pointer's client position when it moves over any row, disabled ones too, so hover and
/// keyboard select alike (O-13) and the menu tracker sees where the pointer is (`point`), its
/// mount, and, where the menu listens for it, a button released over it (`release`).
#[derive(Clone, Copy)]
pub(crate) struct RowEvents {
    /// A click of a live row.
    pub pick: EventHandler<()>,
    /// The pointer moved over the row.
    pub point: EventHandler<Point>,
    /// The row mounted.
    pub mounted: EventHandler<MountedEvent>,
    /// A button was released over the row.
    pub release: Option<EventHandler<Press>>,
}

/// An item row, reporting through `events`.
pub(crate) fn item(view: ItemView<'_>, row: Row, events: RowEvents) -> Element {
    let RowEvents {
        pick: onpick,
        point: onpoint,
        mounted: onmounted,
        release: onrelease,
    } = events;
    let checked = view.check.map(|check| match check {
        Check::Checked => "true",
        Check::Unchecked => "false",
    });
    let title = marked(view.title, view.marks);
    let detail = view.detail.map(str::to_string);
    let (popup, expanded) = match view.branch {
        Branch::Leaf => (None, None),
        Branch::Parent(open) => (Some("true"), Some(open.aria())),
    };
    let trail = match view.branch {
        Branch::Leaf => trail(view.trail, view.check, row),
        Branch::Parent(_) => chevron(),
    };
    let live = view.availability == Availability::Enabled;
    let tile = match row {
        Row::Tiled => Some(tile(view.tile)),
        Row::Checked => None,
    };
    rsx! {
        div {
            class: "ds-menu-item",
            role: "option",
            "aria-selected": view.selection.aria(),
            "aria-checked": checked,
            "aria-disabled": view.availability.aria_disabled(),
            "aria-haspopup": popup,
            "aria-expanded": expanded,
            onmousedown: move |event| event.prevent_default(),
            onmousemove: move |event| {
                event.stop_propagation();
                onpoint.call(client_point(&event));
            },
            onclick: move |_| {
                if live {
                    onpick.call(());
                }
            },
            onmouseup: move |event| {
                if let Some(onrelease) = onrelease {
                    event.stop_propagation();
                    let button = button_of(event.trigger_button()).unwrap_or(PointerButton::Primary);
                    onrelease.call(Press::of(&event, button));
                }
            },
            onmounted: move |event| onmounted.call(event),
            if row == Row::Checked {
                Glyph { icon: Icon::Check }
            }
            {tile}
            span {
                b { class: "ds-menu-title", {title} }
                if let Some(detail) = detail {
                    small { class: "ds-menu-detail ds-truncate", "{detail}" }
                }
            }
            {trail}
        }
    }
}

/// A status line: a row of text at an item's weight, not a choice (design/13 section 13.3.3).
pub(crate) fn info(title: &str, detail: Option<&str>) -> Element {
    let detail = detail.map(str::to_string);
    rsx! {
        div { class: "ds-menu-info", role: "presentation",
            b { class: "ds-menu-info-title", "{title}" }
            if let Some(detail) = detail {
                small { class: "ds-menu-info-detail", "{detail}" }
            }
        }
    }
}

/// Where a mouse event happened, in the client coordinates every quire rect is read in.
pub(crate) fn client_point(event: &MouseEvent) -> Point {
    let at = event.client_coordinates();
    Point {
        x: Px(at.x as f32),
        y: Px(at.y as f32),
    }
}

/// A parent row's trail: the 12 px chevron (design/13 section 13.3.3).
fn chevron() -> Element {
    rsx! {
        span { class: "ds-menu-trail ds-menu-chevron",
            Glyph { icon: Icon::ChevronRight, size: IconSize::Tiny }
        }
    }
}

/// The tile column: a glyph, a letter, an avatar, or an empty square that keeps the grid.
fn tile(tile: Option<&Tile>) -> Element {
    match tile {
        Some(Tile::Icon(icon)) => rsx! {
            span { class: "ds-menu-tile",
                Glyph { icon: *icon, size: IconSize::Tile }
            }
        },
        Some(Tile::Source(source)) => {
            let image = matches!(source, IconSource::Image(_)).then_some("image");
            rsx! {
                span { class: "ds-menu-tile", "data-tile": image,
                    IconView { source: source.clone(), size: IconSize::Tile }
                }
            }
        }
        Some(Tile::Text(text)) => rsx! {
            span { class: "ds-menu-tile", "{text}" }
        },
        Some(Tile::Avatar(avatar)) => rsx! {
            span { class: "ds-menu-tile", "data-tile": "avatar", {face(*avatar)} }
        },
        None => rsx! {
            span { class: "ds-menu-tile", "data-tile": "none" }
        },
    }
}

/// The trail: a checked item in a tiled menu shows the check instead of its shortcut.
fn trail(trail: &Trail, check: Option<Check>, row: Row) -> Element {
    if row == Row::Tiled && check == Some(Check::Checked) {
        return rsx! {
            span { class: "ds-menu-trail",
                Glyph { icon: Icon::Check, size: IconSize::Compact }
            }
        };
    }
    let text = match trail {
        Trail::None => String::new(),
        Trail::Shortcut(shortcut) => shortcut.glyphs(),
        Trail::Note(note) => note.clone(),
    };
    rsx! {
        span { class: "ds-menu-trail", "{text}" }
    }
}
