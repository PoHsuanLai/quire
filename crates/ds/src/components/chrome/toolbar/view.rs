//! Toolbar: the band across the top of a window's content (design/30 section 2.7, `NSToolbar`).
//! Leading items, the title (or a control the caller puts in its place), trailing items, and
//! when they do not all fit the overflow chevron, which opens the rest as a menu. Items are
//! `Button { Toolbar }`: the glyph alone, a fill under the pointer and while pressed. A pick
//! reports the item's value and its button (`Picked`), so a menu or popover can hang from it.
//!
//! Markup: `div.ds-toolbar[role=toolbar][data-overflow]` of `span.ds-toolbar-leading`,
//! `span.ds-toolbar-title` (`.ds-toolbar-heading`, `.ds-toolbar-subtitle`) and
//! `span.ds-toolbar-trailing`.

use crate::components::chrome::toolbar::model::{Kept, Picked, ToolbarItem, ToolbarRoom};
use crate::components::content::icon_source::IconSource;
use crate::components::content::text_runs::{TextLine, text};
use crate::components::controls::button::Button;
use crate::components::controls::button_model::{Bezel, ImagePosition};
use crate::components::menus::item::item::{AfterPick, MenuImage, MenuItem};
use crate::components::menus::menu::menu::Menu;
use crate::components::menus::menu::placement::MenuPlacement;
use crate::host::measure::{Anchor, MountedRef, use_rect};
use crate::root::common::Common;
use dioxus::prelude::*;
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
            key: None,
            hint: None,
            check: item.check,
            availability: item.availability,
            after: AfterPick::Close,
        })
        .collect()
}

/// A toolbar. `leading` and `trailing` are the items either side of the title; `center`
/// replaces the title's words with a control (a segmented control of views). `onpick` hears an
/// item from the band or from the overflow menu, with the button it came from, so a menu can
/// hang from that button. `room` is the width the items are laid out in.
#[component]
pub fn Toolbar<T: Clone + PartialEq + 'static>(
    #[props(default)] leading: Vec<ToolbarItem<T>>,
    #[props(default)] trailing: Vec<ToolbarItem<T>>,
    #[props(default)] title: Option<TextLine>,
    #[props(default)] subtitle: Option<TextLine>,
    #[props(default)] center: Option<Element>,
    #[props(default = ToolbarRoom::Measured)] room: ToolbarRoom,
    onpick: EventHandler<Picked<T>>,
    #[props(default)] common: Common,
) -> Element {
    let buttons: Buttons = use_hook(|| CopyValue::new(HashMap::new()));
    let probe = use_rect();
    let mut open = use_signal(|| Shown::Hidden);
    let mut chevron = use_signal(|| None::<MountedRef>);
    let width = match room {
        ToolbarRoom::Fixed(width) => width,
        ToolbarRoom::Measured => probe.rect().map_or(Px(f32::MAX), |rect| rect.size.width),
    };
    let kept = Kept::fitting(leading.len(), trailing.len(), width);
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
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-toolbar"),
            role: "toolbar",
            "aria-label": common.aria_label.clone(),
            "data-overflow": if overflowing { Some("true") } else { None },
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
            span { class: "ds-toolbar-title",
                if let Some(center) = center {
                    {center}
                } else {
                    if let Some(title) = title.as_ref() {
                        span { class: "ds-toolbar-heading", {text(title)} }
                    }
                    if let Some(subtitle) = subtitle.as_ref() {
                        span { class: "ds-toolbar-subtitle", {text(subtitle)} }
                    }
                }
            }
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
