//! How one menu item draws, shared by `Menu` and `CommandPalette` (design/04-COMPONENTS.md
//! section 20): its tile, its title marked where the query matched, its detail, its trail or
//! submenu chevron, and a row's trailing action. Split from `menu_entry`, which holds the data.

use crate::components::avatar::face;
use crate::components::icon_view::IconView;
use crate::components::menu_entry::{Tile, Trail};
use crate::components::menu_match::marked;
use crate::components::menu_shape;
use crate::components::press::{PointerButton, Press, button_of};
use crate::components::row_action::{RowAction, trailing};
use crate::components::row_shape::RowShape;
use crate::components::text_runs::{Text, text};
use crate::components::vocab::{Availability, Check, Selection, Switch};
use crate::geometry::{Point, Px};
use crate::icon::Icon;
use crate::icon::IconSource;
use crate::icon::render::{Glyph, IconSize};
use dioxus::prelude::*;

/// An item's words as the entry holds them: an `Item`'s string or a `Row`'s runs.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Words<'a> {
    /// A plain string.
    Str(&'a str),
    /// Runs, or a plain `Text`.
    Text(&'a Text),
}

impl Words<'_> {
    /// The words drawn, the title's matched characters marked where it takes marks.
    fn draw(self, marks: &[usize]) -> Element {
        match self {
            Words::Str(plain) => marked(plain, marks),
            Words::Text(Text::Plain(plain)) => marked(plain, marks),
            Words::Text(runs) => text(runs),
        }
    }
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
    pub title: Words<'a>,
    /// One line of help.
    pub detail: Option<Words<'a>>,
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
    /// A button at its end that acts without picking it.
    pub trailing: Option<&'a RowAction>,
    /// How it draws beyond its words (a file, a clipboard entry).
    pub shape: &'a RowShape,
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
    let title = view.title.draw(view.marks);
    let detail = view.detail.map(|detail| detail.draw(&[]));
    let action = view.trailing.map(trailing);
    let (popup, expanded) = match view.branch {
        Branch::Leaf => (None, None),
        Branch::Parent(open) => (Some("true"), Some(open.aria())),
    };
    let trail = match view.branch {
        Branch::Leaf => trail(view.trail, view.check, row, view.shape),
        Branch::Parent(_) => chevron(),
    };
    let live = view.availability == Availability::Enabled;
    let tile = match row {
        Row::Tiled => Some(menu_shape::thumb_tile(view.shape).unwrap_or_else(|| tile(view.tile))),
        Row::Checked => None,
    };
    let words = menu_shape::words(view.shape, title, detail);
    rsx! {
        div {
            class: "ds-menu-item",
            role: "option",
            "aria-selected": view.selection.aria(),
            "aria-checked": checked,
            "aria-disabled": view.availability.aria_disabled(),
            "aria-haspopup": popup,
            "aria-expanded": expanded,
            "data-trailing": view.trailing.map(|_| "action"),
            "data-shape": menu_shape::slug(view.shape),
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
            {words}
            {trail}
            {action}
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

/// The trail: a checked item in a tiled menu shows the check instead of its shortcut; a shaped
/// row leads it with its time.
fn trail(trail: &Trail, check: Option<Check>, row: Row, shape: &RowShape) -> Element {
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
    if *shape != RowShape::Plain {
        return menu_shape::trail(shape, text);
    }
    rsx! {
        span { class: "ds-menu-trail", "{text}" }
    }
}
