//! How one menu item draws (design/30 section 2.4): its state mark, its image, its title, and at
//! its end its key equivalent or the chevron of a submenu. Called by `lines` for each line of a
//! panel.

use crate::components::content::avatar::face;
use crate::components::content::icon_source::IconSource;
use crate::components::content::icon_view::IconView;
use crate::components::controls::press::{button_of, press_of};
use crate::components::menus::alive::Alive;
use crate::components::menus::item::item::MenuImage;
use crate::components::menus::item::text::{Emphasis, ItemText};
use crate::components::menus::menu::placement::Keys;
use dioxus::prelude::*;
use ds_core::geometry::units::{Point, Px};
use ds_core::press::{PointerButton, Press};
use ds_core::vocab::{Availability, Check, FocusStyle, Selection, Shortcut, Shown};
use ds_core::word::Word;
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};

/// Whether an item's panel keeps a column for the state mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StateColumn {
    /// Some item has a state: every item keeps the column so the titles line up.
    Reserved,
    /// None does.
    Absent,
}

/// Whether an item's panel keeps a column for the image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImageColumn {
    /// Some item has an image.
    Reserved,
    /// None does.
    Absent,
}

/// The leading columns a panel keeps, so its titles line up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Columns {
    pub state: StateColumn,
    pub image: ImageColumn,
}

/// Whether an item opens a submenu, and whether that submenu is open (`aria-expanded`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Branch {
    /// It picks a value.
    Leaf,
    /// It opens a submenu, open or not.
    Parent(Shown),
}

/// One item as a menu draws it.
pub(crate) struct ItemView<'a> {
    /// The title.
    pub title: &'a str,
    /// The image.
    pub image: Option<&'a MenuImage>,
    /// What the row says beside its title; a submenu parent says nothing.
    pub text: Option<&'a ItemText>,
    /// The trailing hint.
    pub hint: Option<&'a str>,
    /// The key equivalent.
    pub key: Option<&'a Shortcut>,
    /// The state mark.
    pub check: Option<Check>,
    /// Whether this is the highlighted item.
    pub highlight: Selection,
    /// Whether it can be picked.
    pub availability: Availability,
    /// Whether it opens a submenu.
    pub branch: Branch,
    /// The panel's leading columns.
    pub columns: Columns,
    /// Whether key equivalents show.
    pub keys: Keys,
}

/// What an item reports: a click of a live item (`pick`; a parent opens its submenu), the
/// pointer's client position when it moves over any item, disabled ones too, so the highlight
/// follows the pointer and the menu tracker sees where it is (`point`), its mount, and, where the
/// menu listens for it, a button released over it (`release`).
#[derive(Clone)]
pub(crate) struct RowEvents {
    /// A click of a live item.
    pub pick: EventHandler<()>,
    /// The pointer moved over the item.
    pub point: EventHandler<Point>,
    /// The item mounted.
    pub mounted: EventHandler<MountedEvent>,
    /// A button was released over the item.
    pub release: Option<EventHandler<Press>>,
    /// Whether the menu that owns these handlers is still mounted; every listener asks before
    /// it calls one, since the row can outlive its owner by a frame.
    pub alive: Alive,
}

/// An item, reporting through `events`.
pub(crate) fn item(view: ItemView<'_>, events: RowEvents) -> Element {
    let RowEvents {
        pick: onpick,
        point: onpoint,
        mounted: onmounted,
        release: onrelease,
        alive,
    } = events;
    let (moved, clicked, released, mounted) = (alive.clone(), alive.clone(), alive.clone(), alive);
    let (popup, expanded) = match view.branch {
        Branch::Leaf => (None, None),
        Branch::Parent(open) => (Some("true"), Some(open.aria())),
    };
    let role = match view.check {
        Some(_) => "menuitemcheckbox",
        None => "menuitem",
    };
    let highlighted = (view.highlight == Selection::Selected).then_some("true");
    let live = view.availability == Availability::Enabled;
    let state = match view.columns.state {
        StateColumn::Reserved => Some(state_mark(view.check)),
        StateColumn::Absent => None,
    };
    // Without the state column the title (or image) leads the row and takes the row's own inset.
    let leading = match view.columns.state {
        StateColumn::Reserved => None,
        StateColumn::Absent => Some("absent"),
    };
    let image = match view.columns.image {
        ImageColumn::Reserved => Some(image_mark(view.image)),
        ImageColumn::Absent => None,
    };
    let end = match (view.branch, view.keys, view.key) {
        (Branch::Parent(_), _, _) => chevron(),
        (Branch::Leaf, Keys::Shown, Some(key)) => rsx! {
            span { class: "ds-menu-trailing", "data-mark": "keys", "{key.glyphs()}" }
        },
        (Branch::Leaf, Keys::Hidden, _) | (Branch::Leaf, Keys::Shown, None) => rsx! {},
    };
    let label = label(view.title, view.text);
    let lines = view
        .text
        .is_some_and(|text| text.subtitle.is_some())
        .then_some("two");
    let hint = view.hint.map(|hint| {
        rsx! {
            span { class: "ds-menu-hint", "{hint}" }
        }
    });
    rsx! {
        div {
            class: "ds-menu-item",
            role,
            "data-selected": highlighted,
            "data-focus": FocusStyle::Highlight.slug(),
            "data-state-column": leading,
            "data-lines": lines,
            "aria-checked": view.check.map(Check::aria),
            "aria-disabled": view.availability.aria_disabled(),
            "aria-busy": view.availability.aria_busy(),
            "aria-haspopup": popup,
            "aria-expanded": expanded,
            onmousedown: move |event| event.prevent_default(),
            onmousemove: move |event| {
                event.stop_propagation();
                moved.run(|| onpoint.call(client_point(&event)));
            },
            onclick: move |_| {
                if live {
                    clicked.run(|| onpick.call(()));
                }
            },
            onmouseup: move |event| {
                if let Some(onrelease) = onrelease {
                    event.stop_propagation();
                    let button = button_of(event.trigger_button()).unwrap_or(PointerButton::Primary);
                    released.run(|| onrelease.call(press_of(&event, button)));
                }
            },
            onmounted: move |event| {
                mounted.run(|| onmounted.call(event));
            },
            {state}
            {image}
            {label}
            {hint}
            {end}
        }
    }
}

/// The title: plain text for a plain item, and for one with matched characters or a second line
/// the title's runs over its subtitle.
fn label(title: &str, text: Option<&ItemText>) -> Element {
    let Some(text) = text.filter(|text| !text.marks.is_empty() || text.subtitle.is_some()) else {
        return rsx! {
            span { class: "ds-menu-label", "{title}" }
        };
    };
    let runs = text.marks.runs(title);
    let subtitle = text.subtitle.clone();
    rsx! {
        span { class: "ds-menu-label",
            span { class: "ds-menu-title",
                for (at , run) in runs.into_iter().enumerate() {
                    match run.emphasis {
                        Emphasis::Plain => rsx! {
                            Fragment { key: "{at}", "{run.text}" }
                        },
                        Emphasis::Matched => rsx! {
                            span { key: "{at}", class: "ds-menu-mark", "{run.text}" }
                        },
                    }
                }
            }
            if let Some(subtitle) = subtitle {
                span { class: "ds-menu-subtitle", "{subtitle}" }
            }
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

/// A group title.
pub(crate) fn header(title: &str) -> Element {
    rsx! {
        div { class: "ds-menu-header", role: "presentation", "{title}" }
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

/// A parent row's trailing mark: the 12 px chevron (design/13 section 13.3.3).
fn chevron() -> Element {
    rsx! {
        span { class: "ds-menu-trailing", "data-mark": "chevron",
            Glyph { icon: Icon::ChevronRight, size: IconSize::Tiny }
        }
    }
}

/// The state column: a check for `On`, a dash for `Mixed`, empty for `Off` or no state.
fn state_mark(check: Option<Check>) -> Element {
    let icon = match check {
        Some(Check::On) => Some(Icon::Check),
        Some(Check::Mixed) => Some(Icon::Minus),
        Some(Check::Off) | None => None,
    };
    rsx! {
        span { class: "ds-menu-state",
            if let Some(icon) = icon {
                Glyph { icon, size: IconSize::Compact }
            }
        }
    }
}

/// The image column: a glyph, an icon source, or an empty box that keeps the titles lined up.
fn image_mark(image: Option<&MenuImage>) -> Element {
    match image {
        Some(MenuImage::Icon(icon)) => rsx! {
            span { class: "ds-menu-image",
                Glyph { icon: *icon, size: IconSize::Base }
            }
        },
        Some(MenuImage::Source(source)) => {
            let picture = matches!(source, IconSource::Image(_)).then_some("image");
            rsx! {
                span { class: "ds-menu-image", "data-image": picture,
                    IconView { source: source.clone(), size: IconSize::Base }
                }
            }
        }
        Some(MenuImage::Avatar(avatar)) => rsx! {
            span { class: "ds-menu-image", "data-image": "avatar",
                {face(*avatar)}
            }
        },
        None => rsx! {
            span { class: "ds-menu-image", "data-image": "none" }
        },
    }
}
