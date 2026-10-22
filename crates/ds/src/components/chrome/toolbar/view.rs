//! Toolbar: the band across the top of a window's content (design/30 section 2.7, `NSToolbar`).
//! Leading items, the title (or a control the caller puts in its place), trailing items, and
//! when they do not all fit the overflow chevron, which opens the rest as a menu. Items are
//! `Button { Toolbar }`: the glyph alone, a fill under the pointer and while pressed. A pick
//! reports the item's value and its button (`Picked`), so a menu or popover can hang from it.
//!
//! Markup: `div.ds-toolbar[role=toolbar][data-overflow]` of `span.ds-toolbar-leading`,
//! `span.ds-toolbar-title` (`.ds-toolbar-heading`, `.ds-toolbar-subtitle`),
//! `span.ds-toolbar-center` (only with a title and a centre item together) and
//! `span.ds-toolbar-trailing`.

use crate::components::chrome::path_control::model::{PathSegment, rows};
use crate::components::chrome::toolbar::model::{
    Expansion, Kept, Picked, ToolbarItem, ToolbarRoom, search_fit, search_room,
};
use crate::components::chrome::toolbar::search::{ToolbarSearch, search_item};
use crate::components::content::icon_source::IconSource;
use crate::components::content::text_runs::{TextLine, text};
use crate::components::controls::button::Button;
use crate::components::controls::button_model::{Bezel, ImagePosition};
use crate::components::menus::item::item::{AfterPick, MenuImage, MenuItem};
use crate::components::menus::menu::menu::Menu;
use crate::components::menus::menu::placement::MenuPlacement;
use crate::host::measure::{Anchor, MountedRef, use_rect};
use crate::keys::use_platform;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::command::holds_primary;
use ds_core::geometry::units::Px;
use ds_core::press::Press;
use ds_core::vocab::{Availability, Shown};
use ds_style::icon::Icon;
use ds_style::tokens::control_size::ControlSize;
use std::collections::HashMap;

/// The mounted buttons, by the item's place on the band (leading items first, then trailing).
type Buttons = CopyValue<HashMap<usize, MountedRef>>;

/// One item, at `place` on the band, as a button that hands its own element over when picked.
fn button<T: Clone + PartialEq + 'static>(
    item: &ToolbarItem<T>,
    place: usize,
    buttons: Buttons,
    onpick: EventHandler<Picked<T>>,
) -> Element {
    let value = item.value.clone();
    rsx! {
        Button {
            label: item.label.clone(),
            title: Some(item.label.clone()),
            title_shortcut: item.shortcut.clone(),
            bezel: Bezel::Toolbar,
            size: ControlSize::Large,
            image: ImagePosition::Only,
            icon: Some(IconSource::from(item.icon)),
            value: item.check,
            availability: item.availability,
            onclick: move |_: Press| {
                let anchor = buttons.peek().get(&place).cloned().map(Anchor::Mounted);
                onpick.call(Picked { value: value.clone(), anchor });
            },
            common: Common {
                mounted: Some(EventHandler::new(move |event: MountedEvent| {
                    let mut buttons = buttons;
                    buttons.write().insert(place, MountedRef(event.data()));
                })),
                ..Common::default()
            },
        }
    }
}

/// The overflow menu's rows for the items that did not fit.
fn hidden_rows<T: Clone>(items: &[&ToolbarItem<T>]) -> Vec<MenuItem<T>> {
    items
        .iter()
        .map(|item| MenuItem::Item {
            value: item.value.clone(),
            title: item.label.clone(),
            image: Some(MenuImage::Icon(item.icon)),
            key: item.shortcut.clone(),
            hint: None,
            check: item.check,
            availability: item.availability,
            after: AfterPick::Close,
            text: Default::default(),
        })
        .collect()
}

/// What the title's span does when the toolbar has a title menu: a click on it, and its
/// element, which the menu hangs from.
#[derive(Clone, Copy)]
struct TitleMenu {
    onclick: EventHandler<MouseEvent>,
    onmounted: EventHandler<MountedEvent>,
}

/// The title's span, and the centre item's beside it when both are given. With only a centre
/// item that item takes the title's span.
fn title_span(
    title: Option<&TextLine>,
    subtitle: Option<&TextLine>,
    center: Option<Element>,
    menu: Option<TitleMenu>,
) -> Element {
    let onclick = move |event: MouseEvent| {
        if let Some(menu) = menu {
            menu.onclick.call(event);
        }
    };
    let onmounted = move |event: MountedEvent| {
        if let Some(menu) = menu {
            menu.onmounted.call(event);
        }
    };
    let words = rsx! {
        if let Some(title) = title {
            span { class: "ds-toolbar-heading", {text(title)} }
        }
        if let Some(subtitle) = subtitle {
            span { class: "ds-toolbar-subtitle", {text(subtitle)} }
        }
    };
    match (title.is_some() || subtitle.is_some(), center) {
        (false, Some(center)) => rsx! {
            span {
                class: "ds-toolbar-title",
                onclick,
                onmounted,
                {center}
            }
        },
        (true, Some(center)) => rsx! {
            span {
                class: "ds-toolbar-title",
                "data-with-center": "true",
                onclick,
                onmounted,
                {words}
            }
            span { class: "ds-toolbar-center", {center} }
        },
        (_, None) => rsx! {
            span {
                class: "ds-toolbar-title",
                onclick,
                onmounted,
                {words}
            }
        },
    }
}

/// A toolbar. `leading` and `trailing` are the items either side of the title; `center` is a
/// control (a segmented control of views). With no title it takes the title's place, centred;
/// with a title the two sit together as in Finder, the title after the leading items and the
/// control centred in the room that is left. A button's tooltip carries its item's shortcut
/// (`Back  ⌘[`). `oncontextmenu` hears a right click anywhere on the band and keeps it from
/// reaching the window's own menu behind it. `onpick` hears an
/// item from the band or from the overflow menu, with the button it came from, so a menu can
/// hang from that button. `room` is the width the items are laid out in. `search` is a search
/// field at the band's trailing end that collapses to a magnifier button when the band is too
/// tight for it (`ToolbarSearch`); the field gives way before any item goes behind the chevron.
///
/// `title_menu` makes the title a place that opens a menu: a click on it with the primary
/// modifier held (Command on a Mac or our desktop, Ctrl elsewhere) lists the segments, which are
/// the folders enclosing the one the title names, in the order the caller gives; `ontitlemenu`
/// hears the one picked, by its index in `title_menu`. A plain click on the title does nothing.
/// With an empty `title_menu` the title is plain text.
#[component]
pub fn Toolbar<T: Clone + PartialEq + 'static>(
    #[props(default)] leading: Vec<ToolbarItem<T>>,
    #[props(default)] trailing: Vec<ToolbarItem<T>>,
    #[props(default)] title: Option<TextLine>,
    #[props(default)] subtitle: Option<TextLine>,
    #[props(default)] center: Option<Element>,
    #[props(default = ToolbarRoom::Measured)] room: ToolbarRoom,
    #[props(default)] search: Option<ToolbarSearch>,
    #[props(default)] title_menu: Vec<PathSegment>,
    #[props(default)] ontitlemenu: Option<EventHandler<usize>>,
    #[props(default)] oncontextmenu: Option<EventHandler<MouseEvent>>,
    onpick: EventHandler<Picked<T>>,
    #[props(default)] common: Common,
) -> Element {
    let buttons: Buttons = use_hook(|| CopyValue::new(HashMap::new()));
    let probe = use_rect();
    let mut open = use_signal(|| Shown::Hidden);
    let mut chevron = use_signal(|| None::<MountedRef>);
    let platform = use_platform();
    let mut title_open = use_signal(|| Shown::Hidden);
    let mut title_seat = use_signal(|| None::<MountedRef>);
    let width = match room {
        ToolbarRoom::Fixed(width) => width,
        ToolbarRoom::Measured => probe.rect().map_or(Px(f32::MAX), |rect| rect.size.width),
    };
    let expansion = use_signal(Expansion::default);
    let fit = search
        .as_ref()
        .map(|search| search_fit(leading.len(), trailing.len(), width, search.min_width));
    // The search item takes its room first, so the items overflow around it.
    let taken = search.as_ref().zip(fit).map_or(Px(0.0), |(search, fit)| {
        search_room(fit, expansion(), search.min_width)
    });
    let kept = Kept::fitting(leading.len(), trailing.len(), Px(width.0 - taken.0));
    let overflowing = !kept.all(leading.len(), trailing.len());
    let hidden: Vec<&ToolbarItem<T>> = leading
        .iter()
        .skip(kept.leading)
        .chain(trailing.iter().skip(kept.trailing))
        .collect();
    let rows = hidden_rows(&hidden);
    let mounted = common.clone();
    let data = common.data_attributes();
    let anchor = chevron().map(Anchor::Mounted);
    let title_anchor = title_seat().map(Anchor::Mounted);
    let menu = (!title_menu.is_empty()).then(|| TitleMenu {
        onclick: EventHandler::new(move |event: MouseEvent| {
            if holds_primary(platform, event.modifiers()) {
                title_open.set(Shown::Visible);
            }
        }),
        onmounted: EventHandler::new(move |event: MountedEvent| {
            title_seat.set(Some(MountedRef(event.data())));
        }),
    });
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-toolbar"),
            role: "toolbar",
            "aria-label": common.aria_label.clone(),
            "data-overflow": if overflowing { Some("true") } else { None },
            oncontextmenu: move |event: MouseEvent| {
                if let Some(hear) = oncontextmenu {
                    event.stop_propagation();
                    hear.call(event);
                }
            },
            onmounted: move |event| {
                probe.on_mounted(event.clone());
                mounted.mounted(event);
            },
            ..data,
            span { class: "ds-toolbar-leading",
                for (place , item) in leading.iter().enumerate().take(kept.leading) {
                    {button(item, place, buttons, onpick)}
                }
            }
            {title_span(title.as_ref(), subtitle.as_ref(), center, menu)}
            span { class: "ds-toolbar-trailing",
                for (at , item) in trailing.iter().enumerate().take(kept.trailing) {
                    {button(item, leading.len() + at, buttons, onpick)}
                }
                if overflowing {
                    Button {
                        label: "More",
                        title: Some("More".to_owned()),
                        bezel: Bezel::Toolbar,
                        size: ControlSize::Large,
                        image: ImagePosition::Only,
                        icon: Some(IconSource::from(Icon::ChevronRight)),
                        shown: Some(open()),
                        availability: Availability::Enabled,
                        onclick: move |_: Press| open.set(open().flipped()),
                        common: Common {
                            mounted: Some(EventHandler::new(move |event: MountedEvent| {
                                chevron.set(Some(MountedRef(event.data())));
                            })),
                            ..Common::default()
                        },
                    }
                }
                if let (Some(search), Some(fit)) = (search.as_ref(), fit) {
                    {search_item(search, fit, expansion)}
                }
            }
        }
        if title_open() == Shown::Visible && let Some(anchor) = title_anchor {
            Menu::<usize> {
                placement: MenuPlacement::Popup,
                anchor,
                items: rows(&title_menu, 0),
                onpick: move |at: usize| {
                    title_open.set(Shown::Hidden);
                    if let Some(hear) = ontitlemenu {
                        hear.call(at);
                    }
                },
                onclose: move |()| title_open.set(Shown::Hidden),
            }
        }
        if open() == Shown::Visible && let Some(anchor) = anchor {
            Menu::<T> {
                placement: MenuPlacement::Popup,
                anchor,
                items: rows,
                onpick: move |value: T| {
                    open.set(Shown::Hidden);
                    onpick.call(Picked { value, anchor: chevron().map(Anchor::Mounted) });
                },
                onclose: move |()| open.set(Shown::Hidden),
            }
        }
    }
}
