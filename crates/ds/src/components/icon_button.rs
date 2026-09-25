//! IconButton: an icon-only action, `aria-label` mandatory (design/04-COMPONENTS.md section 2).

use crate::components::button_size::disabled;
use crate::components::icon_view::IconView;
use crate::components::press::{Press, PressListeners, Propagation};
use crate::components::vocab::{Availability, Switch};
use crate::geometry::Px;
use crate::icon::external::IconSource;
use crate::icon::render::IconSize;
use crate::tokens::VarName;
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
    /// A bar status item on the frame (design/13-BEHAVIOUR-menus-windows.md section 13.3.1):
    /// a square of `--bar-status-box` holding a glyph of `--bar-status-glyph`, both set by the
    /// consumer from its settings ([`StatusMetrics`]); `--f-ink-soft` at rest, `--f-ink` on
    /// `--f-pill-hover` under the pointer, on `--f-pill` pressed or while its menu is open.
    Status,
}

impl IconButtonVariant {
    /// The `data-variant` word.
    fn slug(self) -> &'static str {
        match self {
            IconButtonVariant::Tool => "tool",
            IconButtonVariant::Foot => "foot",
            IconButtonVariant::Strip => "strip",
            IconButtonVariant::Pin => "pin",
            IconButtonVariant::Status => "status",
        }
    }

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
/// `mounted` hands over the element once it is in the document, so a floating component can
/// anchor to it (`Anchor::Mounted`). `propagation: Propagation::Stop` keeps the press at the
/// button, so a glyph inside a `<summary>` does not toggle its `<details>`.
/// `availability: Availability::Disabled` writes `aria-disabled` and `disabled` and draws the
/// button at .35 with no hover and no press; `onclick` never runs (sill Q93).
#[component]
pub fn IconButton(
    variant: IconButtonVariant,
    #[props(into)] icon: IconSource,
    label: String,
    #[props(default)] tooltip: Option<String>,
    #[props(default)] pressed: Option<Switch>,
    #[props(default)] expanded: Option<Switch>,
    #[props(default)] availability: Availability,
    onclick: EventHandler<Press>,
    #[props(default)] id: Option<String>,
    #[props(default)] mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)] propagation: Propagation,
) -> Element {
    let pressed = pressed.map(|state| state.aria());
    let expanded = expanded.map(|state| state.aria());
    let listen = PressListeners::new(onclick).with_propagation(propagation);
    let live = availability == Availability::Enabled;
    rsx! {
        button {
            r#type: "button",
            id,
            class: "ds-icon-button",
            "data-variant": variant.slug(),
            "aria-label": "{label}",
            title: tooltip,
            "aria-pressed": pressed,
            "aria-expanded": expanded,
            "aria-disabled": availability.aria_disabled(),
            disabled: disabled(availability),
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
            IconView { source: icon, size: variant.icon_size() }
        }
    }
}

/// A bar's status item geometry, from the shell's settings (design/22-SETTINGS.md section 3,
/// `bar.status_icon_box_px`, `bar.status_glyph_px`, `bar.glyph_size_policy`): written as the
/// two custom properties `IconButton { Status }` reads, on any element around the items.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatusMetrics {
    /// The item's square box (`bar.status_icon_box_px`, default 22).
    pub box_size: Px,
    /// The glyph inside it (`bar.status_glyph_px`, 16 by default; the box itself under the
    /// `IconSizeBar22` glyph policy).
    pub glyph: Px,
}

impl StatusMetrics {
    /// The property the box is read from.
    pub const BOX_VAR: VarName = VarName("--bar-status-box");
    /// The property the glyph is read from.
    pub const GLYPH_VAR: VarName = VarName("--bar-status-glyph");

    /// The inline declarations: `--bar-status-box:22px;--bar-status-glyph:16px;`.
    pub fn style_attr(&self) -> String {
        format!(
            "{}:{}px;{}:{}px;",
            Self::BOX_VAR.as_str(),
            self.box_size.0,
            Self::GLYPH_VAR.as_str(),
            self.glyph.0
        )
    }
}

impl Default for StatusMetrics {
    /// The keys' defaults: a 22 px box and a 16 px glyph.
    fn default() -> Self {
        StatusMetrics {
            box_size: Px(22.0),
            glyph: Px(16.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StatusMetrics;
    use crate::geometry::Px;

    #[test]
    fn the_metrics_write_both_properties() {
        assert_eq!(
            StatusMetrics::default().style_attr(),
            "--bar-status-box:22px;--bar-status-glyph:16px;"
        );
        let full = StatusMetrics {
            box_size: Px(24.0),
            glyph: Px(24.0),
        };
        assert_eq!(
            full.style_attr(),
            "--bar-status-box:24px;--bar-status-glyph:24px;"
        );
    }
}
