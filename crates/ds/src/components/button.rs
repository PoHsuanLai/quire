//! Button: a labelled action in five variants (design/04-COMPONENTS.md section 1).
//! Markup: `button.ds-button[data-variant]`, `aria-pressed` only for a toggle Mini.

use crate::components::icon_view::IconView;
use crate::components::press::{Press, PressListeners};
use crate::components::vocab::{Availability, Switch};
use crate::icon::external::IconSource;
use crate::icon::render::IconSize;
use dioxus::prelude::*;

/// Which button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonVariant {
    /// Accent fill: Send, Compose.
    Primary,
    /// Surface-2 with a line: the ghost button.
    Secondary,
    /// Small, surface-2: Reply, RSVP, hover-card actions.
    Mini,
    /// Text only: the quiet link-button.
    Quiet,
    /// As Mini at rest; red only on hover.
    Danger,
}

impl ButtonVariant {
    /// The `data-variant` word.
    fn slug(self) -> &'static str {
        match self {
            ButtonVariant::Primary => "primary",
            ButtonVariant::Secondary => "secondary",
            ButtonVariant::Mini => "mini",
            ButtonVariant::Quiet => "quiet",
            ButtonVariant::Danger => "danger",
        }
    }

    /// The icon size: 14 for every variant that the doc sizes. TODO(O-3): the Quiet icon is
    /// "not specified" in design/04-COMPONENTS.md section 1; it takes the same 14.
    fn icon_size(self) -> IconSize {
        IconSize::Compact
    }
}

/// A labelled action. `icon` is a glyph or an external icon (an `Icon` or `Option<Icon>`
/// converts). `id` is written as the element's `id`, so a popup can anchor to it by id.
/// `onclick` hears the primary, secondary (right-click) and middle buttons, and the keyboard as
/// primary. `mounted` hands over the element once it is in the document, so a floating
/// component can anchor to it (`Anchor::Mounted`).
#[component]
pub fn Button(
    variant: ButtonVariant,
    label: String,
    #[props(default)] icon: Option<IconSource>,
    #[props(default)] pressed: Option<Switch>,
    #[props(default)] availability: Availability,
    onclick: EventHandler<Press>,
    #[props(default)] id: Option<String>,
    #[props(default)] mounted: Option<EventHandler<MountedEvent>>,
) -> Element {
    let pressed = pressed.map(|state| state.aria());
    let listen = PressListeners::new(onclick);
    let live = availability == Availability::Enabled;
    rsx! {
        button {
            r#type: "button",
            id,
            class: "ds-button",
            "data-variant": variant.slug(),
            "aria-pressed": pressed,
            "aria-disabled": availability.aria_disabled(),
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
                if live {
                    listen.mouse_up(&event);
                }
            },
            // The element, for a menu or popover anchored to it (`Anchor::Mounted`). No
            // attribute: the markup is the same with or without a handler.
            onmounted: move |event| {
                if let Some(mounted) = mounted {
                    mounted.call(event);
                }
            },
            if let Some(icon) = icon {
                IconView { source: icon, size: variant.icon_size() }
            }
            span { "{label}" }
        }
    }
}
