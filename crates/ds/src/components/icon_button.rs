//! IconButton: an icon-only action, `aria-label` mandatory (design/04-COMPONENTS.md section 2).

use crate::components::vocab::{Availability, Switch};
use crate::icon::Icon;
use crate::icon::render::{Glyph, IconSize};
use dioxus::prelude::*;

/// Which icon button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconButtonVariant {
    /// Reader and composer tools, 28 x 26.
    Tool,
    /// Sidebar foot, 24 x 24, on the frame.
    Foot,
    /// A row's hover strip, 26 x 26 round.
    Strip,
    /// A square tile on the frame (account tiles).
    Pin,
}

impl IconButtonVariant {
    /// The `data-variant` word.
    fn slug(self) -> &'static str {
        match self {
            IconButtonVariant::Tool => "tool",
            IconButtonVariant::Foot => "foot",
            IconButtonVariant::Strip => "strip",
            IconButtonVariant::Pin => "pin",
        }
    }

    /// The glyph size from the section's geometry table: Tool and Foot 16, Strip 14. The Pin's
    /// content is the account tile's (section 27); a bare glyph on a Pin takes the base 16.
    fn icon_size(self) -> IconSize {
        match self {
            IconButtonVariant::Tool | IconButtonVariant::Foot | IconButtonVariant::Pin => {
                IconSize::Base
            }
            IconButtonVariant::Strip => IconSize::Compact,
        }
    }
}

/// An icon-only action. `mounted` hands over the element once it is in the document, so a
/// floating component can anchor to it (`Anchor::Mounted`).
#[component]
pub fn IconButton(
    variant: IconButtonVariant,
    icon: Icon,
    label: String,
    #[props(default)] tooltip: Option<String>,
    #[props(default)] pressed: Option<Switch>,
    #[props(default)] expanded: Option<Switch>,
    #[props(default)] availability: Availability,
    onclick: EventHandler<()>,
    #[props(default)] mounted: Option<EventHandler<MountedEvent>>,
) -> Element {
    let pressed = pressed.map(|state| state.aria());
    let expanded = expanded.map(|state| state.aria());
    rsx! {
        button {
            r#type: "button",
            class: "ds-icon-button",
            "data-variant": variant.slug(),
            "aria-label": "{label}",
            title: tooltip,
            "aria-pressed": pressed,
            "aria-expanded": expanded,
            "aria-disabled": availability.aria_disabled(),
            onclick: move |_| {
                if availability == Availability::Enabled {
                    onclick.call(());
                }
            },
            // The element, for a menu or popover anchored to it (`Anchor::Mounted`). No
            // attribute: the markup is the same with or without a handler.
            onmounted: move |event| {
                if let Some(mounted) = mounted {
                    mounted.call(event);
                }
            },
            Glyph { icon, size: variant.icon_size() }
        }
    }
}
