//! Button: a labelled action in five variants (design/04-COMPONENTS.md section 1).
//! Markup: `button.ds-button[data-variant]`, `aria-pressed` only for a toggle Mini.

use crate::components::vocab::{Availability, Switch};
use crate::icon::Icon;
use crate::icon::render::{Glyph, IconSize};
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

/// A labelled action.
#[component]
pub fn Button(
    variant: ButtonVariant,
    label: String,
    #[props(default)] icon: Option<Icon>,
    #[props(default)] pressed: Option<Switch>,
    #[props(default)] availability: Availability,
    onclick: EventHandler<()>,
) -> Element {
    let pressed = pressed.map(|state| state.aria());
    rsx! {
        button {
            r#type: "button",
            class: "ds-button",
            "data-variant": variant.slug(),
            "aria-pressed": pressed,
            "aria-disabled": availability.aria_disabled(),
            onclick: move |_| {
                if availability == Availability::Enabled {
                    onclick.call(());
                }
            },
            if let Some(icon) = icon {
                Glyph { icon, size: variant.icon_size() }
            }
            span { "{label}" }
        }
    }
}
