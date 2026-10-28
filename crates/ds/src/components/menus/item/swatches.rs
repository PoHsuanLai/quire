//! A row of swatches as a menu draws it (`MenuItem::Swatches`): Finder's tag colours on one line,
//! each dot its own pick target with a tooltip and an `aria-label` of the tag's name. The dots are
//! consecutive choices of the panel, so the pointer, a click and the highlight reach one dot at a
//! time; Left and Right move across them (`menu::decide`).

use crate::components::content::tip_text::TipText;
use crate::components::controls::press::{button_of, press_of};
use crate::components::menus::item::tip::RowTip;
use crate::components::menus::item::view::RowEvents;
use crate::host::event_point::client_point;
use dioxus::prelude::*;
use ds_core::press::PointerButton;
use ds_core::vocab::{Availability, Check, FocusStyle, Selection};
use ds_core::word::Word;
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};
use ds_style::tokens::system_hue::SystemHue;

/// One dot as a menu draws it.
pub(crate) struct DotView<'a> {
    /// The tag's name.
    pub name: &'a str,
    /// Its colour.
    pub colour: SystemHue,
    /// Whether the tag is on the files.
    pub check: Option<Check>,
    /// Whether this is the highlighted dot.
    pub highlight: Selection,
    /// Whether it can be picked.
    pub availability: Availability,
}

/// The row: `dots` in order, each with the choice number it is and the events it reports.
pub(crate) fn swatches(title: &str, dots: Vec<(usize, DotView<'_>, RowEvents)>) -> Element {
    let drawn: Vec<(usize, Element)> = dots
        .into_iter()
        .map(|(at, view, events)| (at, dot(view, events)))
        .collect();
    rsx! {
        div { class: "ds-menu-swatches", role: "group", "aria-label": "{title}",
            for (at , one) in drawn {
                Fragment { key: "{at}", {one} }
            }
        }
    }
}

/// One dot, reporting through `events` as an item does.
fn dot(view: DotView<'_>, events: RowEvents) -> Element {
    let RowEvents {
        pick: onpick,
        point: onpoint,
        mounted: onmounted,
        release: onrelease,
        alive,
    } = events;
    let (moved, clicked, released, mounted) = (alive.clone(), alive.clone(), alive.clone(), alive);
    let live = view.availability == Availability::Enabled;
    let role = match view.check {
        Some(_) => "menuitemcheckbox",
        None => "menuitem",
    };
    let mark = match view.check {
        Some(Check::On) => Some(Icon::Check),
        Some(Check::Mixed) => Some(Icon::Minus),
        Some(Check::Off) | None => None,
    };
    let highlighted = (view.highlight == Selection::Selected).then_some("true");
    let tip = TipText::new(view.name.to_owned());
    rsx! {
        div {
            class: "ds-menu-dot",
            role,
            "data-selected": highlighted,
            "data-focus": FocusStyle::Highlight.slug(),
            "aria-label": view.name.to_owned(),
            "aria-checked": view.check.map(Check::aria),
            "aria-disabled": view.availability.aria_disabled(),
            "aria-busy": view.availability.aria_busy(),
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
            span { class: "ds-menu-swatch", "data-tag": view.colour.slug(),
                if let Some(icon) = mark {
                    Glyph { icon, size: IconSize::Micro }
                }
            }
            RowTip { tip }
        }
    }
}
