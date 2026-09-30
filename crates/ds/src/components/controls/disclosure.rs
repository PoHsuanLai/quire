//! Disclosure: the triangle that opens and closes a body of content (`NSButton` disclosure
//! triangle, design/30 section 2.1). The triangle turns over `--t-quick`; the body's height and
//! opacity follow it over `--t-move` (`use_collapse`), from the height the body measured.

use crate::host::measure::client_rect;
use crate::root::common::Common;
use dioxus::core::current_scope_id;
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::vocab::{Availability, Shown};
use ds_core::word::Word;
use ds_motion::use_collapse::use_collapse;
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};
use ds_style::task::spawn_in;
use ds_style::tokens::control_size::ControlSize;
use std::rc::Rc;

/// The triangle, turned as `shown` says. A part: `Disclosure` and `Row` draw the same one.
pub(crate) fn indicator(shown: Shown) -> Element {
    rsx! {
        span {
            class: "ds-disclosure-indicator",
            "data-shown": shown.slug(),
            "aria-hidden": "true",
            Glyph { icon: Icon::ChevronRight, size: IconSize::Tiny }
        }
    }
}

/// A body that opens and closes with `shown`: its height and opacity move over `--t-move`, and
/// nothing of it is reachable once it is closed. Measured on mount and whenever `shown` changes,
/// so the content's own height is what it opens to.
#[component]
pub(crate) fn Collapsing(
    shown: Shown,
    #[props(default)] id: Option<String>,
    children: Element,
) -> Element {
    let collapse = use_collapse(shown);
    let mut content = use_signal(|| None::<Rc<MountedData>>);
    let mut measured = use_signal(|| Px(0.0));
    let scope = use_hook(current_scope_id);
    use_effect(use_reactive!(|shown| {
        let _ = shown;
        if let Some(element) = content.peek().clone() {
            spawn_in(scope, async move {
                if let Some(rect) = client_rect(&element).await {
                    measured.set(rect.size.height);
                }
            });
        }
    }));
    let closed = collapse.is_closed().then_some("true");
    rsx! {
        div {
            class: "ds-disclosure-body",
            id,
            role: "region",
            "data-shown": shown.slug(),
            "data-closed": closed,
            "aria-hidden": if shown == Shown::Hidden { Some("true") } else { None },
            style: collapse.style(measured()),
            div {
                class: "ds-disclosure-content",
                onmounted: move |event| {
                    let element = event.data();
                    content.set(Some(element.clone()));
                    spawn_in(scope, async move {
                        if let Some(rect) = client_rect(&element).await {
                            measured.set(rect.size.height);
                        }
                    });
                },
                {children}
            }
        }
    }
}

/// A disclosure: a triangle with an optional label, and the `children` it opens. `on_toggle`
/// hears the state a press asks for; the caller's `shown` decides, so nothing changes until it
/// does. A `Busy` disclosure takes no press.
#[component]
pub fn Disclosure(
    shown: Shown,
    on_toggle: EventHandler<Shown>,
    #[props(default)] label: Option<String>,
    #[props(default)] size: ControlSize,
    #[props(default)] availability: Availability,
    #[props(default)] children: Element,
    #[props(default)] common: Common,
) -> Element {
    let live = availability == Availability::Enabled;
    let data = common.data_attributes();
    let name = common.aria_label.clone().or_else(|| label.clone());
    let body = format!(
        "{}-body",
        common
            .id
            .clone()
            .unwrap_or_else(|| "ds-disclosure".to_owned())
    );
    let controls = body.clone();
    rsx! {
        div {
            class: common.class("ds-disclosure"),
            id: common.id.clone(),
            "data-size": size.slug(),
            "data-shown": shown.slug(),
            ..data,
            button {
                r#type: "button",
                class: "ds-disclosure-trigger",
                "aria-expanded": shown.aria(),
                "aria-controls": "{controls}",
                "aria-label": name,
                "aria-disabled": availability.aria_disabled(),
                "aria-busy": availability.aria_busy(),
                onmounted: move |event| common.mounted(event),
                onclick: move |_| {
                    if live {
                        on_toggle.call(shown.flipped());
                    }
                },
                {indicator(shown)}
                if let Some(label) = label {
                    span { class: "ds-disclosure-label", "{label}" }
                }
            }
            Collapsing { shown, id: body, {children} }
        }
    }
}
