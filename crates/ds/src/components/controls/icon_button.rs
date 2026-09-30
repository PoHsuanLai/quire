//! IconButton: an icon-only action, `aria-label` mandatory (design/04-COMPONENTS.md section 2).

use crate::components::content::icon_source::IconSource;
use crate::components::content::icon_view::IconView;
use crate::components::controls::button_size::disabled;
use crate::components::controls::press::{PressListeners, Propagation, use_pressing};
use crate::root::common::Common;
use crate::style::icon::render::IconSize;
use dioxus::prelude::*;
use ds_core::press::Press;
use ds_core::vocab::{Availability, Check, Shown};
use ds_core::word::Word;

/// Which icon button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum IconButtonVariant {
    /// Reader and composer tools, 28 x 28 (a Large control).
    Tool,
    /// Sidebar foot, 22 x 22 (a Regular control), on the frame.
    Foot,
    /// A row's hover strip, 26 x 26 round (mailo's, off the ladder until the app side is audited).
    Strip,
    /// A square tile on the frame (account tiles).
    Pin,
    /// A bar status item on the frame (design/13-BEHAVIOUR-menus-windows.md section 13.3.1):
    /// a slot `--bar-status-w` (30) wide and `--bar-status-box` tall holding a glyph of `--bar-status-glyph`, both set by the
    /// consumer from its settings ([`StatusMetrics`]); `--f-ink-soft` at rest, `--f-ink` on
    /// `--f-pill-hover` under the pointer, on `--f-pill` pressed or while its menu is open.
    Status,
}

impl IconButtonVariant {
    /// The glyph size from the section's geometry table: Tool and Foot 16, Strip 14. The Pin's
    /// content is the account tile's (section 27); a bare glyph on a Pin takes the base 16. A
    /// Status glyph is written at the base 16 and sized by `--bar-status-glyph` in the sheet.
    fn icon_size(self) -> IconSize {
        match self {
            IconButtonVariant::Tool
            | IconButtonVariant::Foot
            | IconButtonVariant::Pin
            | IconButtonVariant::Status => IconSize::Base,
            IconButtonVariant::Strip => IconSize::Compact,
        }
    }
}

/// An icon-only action. `icon` is a glyph or an external icon (an `Icon` converts). `id` is
/// written as the element's `id`, so a popup can anchor to it by id. `onclick` hears the
/// primary, secondary (right-click) and middle buttons, and the keyboard as primary.
/// `common.mounted` hands over the element once it is in the document, so a floating component
/// can anchor to it (`Anchor::Mounted`). `propagation: Propagation::Stop` keeps the press at the
/// button, so a glyph inside a `<summary>` does not toggle its `<details>`.
/// `availability: Availability::Disabled` writes `aria-disabled` and `disabled` and draws the
/// button at .35 with no hover and no press; `onclick` never runs.
///
/// `common` puts the consumer's own `id`, `data-*` attributes and classes on the button itself,
/// as on a `Button`, so it needs no wrapping `span` (see [`Common`]); its `aria_label` replaces
/// `label` as the accessible name.
///
/// A status item's glyph can be a layered status glyph: `icon: IconSource::Status(state)` (a
/// `StatusState` converts) draws `StatusGlyph` in the same box, ink, pill and label as an `Icon`,
/// sized by `--bar-status-glyph`; hand it the state every render and it plays its own
/// moments.
#[component]
pub fn IconButton(
    variant: IconButtonVariant,
    #[props(into)] icon: IconSource,
    label: String,
    #[props(default)] tooltip: Option<String>,
    #[props(default)] pressed: Option<Check>,
    #[props(default)] expanded: Option<Shown>,
    #[props(default)] availability: Availability,
    onclick: EventHandler<Press>,
    #[props(default)] propagation: Propagation,
    #[props(default)] common: Common,
) -> Element {
    let class = common.class("ds-icon-button");
    let size = variant.icon_size();
    let data = common.data_attributes();
    let pressed = pressed.map(|state| state.aria());
    let expanded = expanded.map(|state| state.aria());
    let listen = PressListeners::new(onclick).with_propagation(propagation);
    let pressing = use_pressing();
    let live = availability == Availability::Enabled;
    rsx! {
        button {
            r#type: "button",
            id: common.id.clone(),
            class,
            "data-variant": variant.slug(),
            "aria-label": common.aria_label.clone().unwrap_or(label.clone()),
            title: tooltip,
            "aria-pressed": pressed,
            "aria-expanded": expanded,
            "aria-disabled": availability.aria_disabled(),
            "aria-busy": availability.aria_busy(),
            disabled: disabled(availability),
            "data-pressed": if live { pressing.attr() } else { None },
            onmousedown: move |event| pressing.pointer_down(&event),
            onmouseleave: move |_| pressing.released(),
            onkeydown: move |event| pressing.key_down(&event),
            onkeyup: move |_| pressing.released(),
            onblur: move |_| pressing.released(),
            onclick: move |event| {
                if live {
                    listen.click(&event);
                }
            },
            oncontextmenu: move |event| {
                if live {
                    listen.context_menu(&event);
                }
            },
            onmouseup: move |event| {
                pressing.released();
                if live {
                    listen.mouse_up(&event);
                }
            },
            // The element, for a menu or popover anchored to it (`Anchor::Mounted`). No
            // attribute: the markup is the same with or without a handler.
            onmounted: move |event| common.mounted(event),
            // The consumer's own `data-*`, last: a spread follows the named
            // attributes.
            ..data,
            IconView { source: icon, size }
        }
    }
}
