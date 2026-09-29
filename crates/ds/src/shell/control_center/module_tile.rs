//! ModuleTile: one control-center module, a glyph disc with its title and status on a tile, and
//! the chevron that opens the module's detail pane (design/20-SURFACES.md
//! section 1.5, design/13-BEHAVIOUR-menus-windows.md section 13.3.7).
//!
//! The tile is a `div[role=button]` rather than a `button`, because the chevron inside it is a
//! button of its own and a button may not hold another. The chevron keeps its press
//! (`Propagation::Stop`), so a press on it opens the detail and never also toggles the module.
//! Keys are handled here on both renderers: Blitz synthesises no click from Enter, and the
//! chevron prevents the default of the keys it takes, so a browser's synthesised click cannot
//! run the same press twice.

use crate::components::content::icon_source::IconSource;
use crate::components::content::text_runs::{TextLine, text};
use crate::components::controls::press::{PressListeners, Propagation};
use crate::core::press::Press;
use crate::core::vocab::{Availability, Expanded};
use crate::core::word::Word;
use crate::focus::click::kept_click;
use crate::motion::detail::{armed::use_armed, first_show::FirstShow, touch::Touch};
use crate::shell::control_center::module_disc::ModuleDisc;
use crate::shell::control_center::module_tile_kind::{Chevron, DiscMotion, ModuleState, TileSpan};
use crate::style::icon::Icon;
use crate::style::icon::render::{Glyph, IconSize};
use dioxus::prelude::*;

/// A control-center module. `onclick` toggles it (a press on the tile, Enter or Space);
/// `on_detail` opens its detail pane (a press on the chevron, Enter or Right on the chevron).
/// `expanded` is the chevron's `aria-expanded`: whether that pane is showing.
///
/// `glyph` is an `Icon` (it converts) or any [`IconSource`]: `IconSource::Status` puts a layered
/// status glyph in the disc (the Wi-Fi and Bluetooth modules), which plays its own
/// moments as the state changes, the Wi-Fi fan filling as it joins. `first` is the glyph's first
/// frame: `FirstShow::Animate` when the control center was just opened.
///
/// `disc` is how the disc's glyph answers the module coming on (design/26): `Still`
/// (the default), `Fill` (the glyph's layers fill once, the Wi-Fi fan) or `Morph(icon)` (the
/// glyph grows into `icon`, the Focus moon into `Icon::MoonFilled`). A press on the tile is kept
/// for the change it causes, so that change springs and one from elsewhere does not (R5).
#[component]
pub fn ModuleTile(
    #[props(into)] glyph: IconSource,
    #[props(into)] title: TextLine,
    status: Option<TextLine>,
    state: ModuleState,
    #[props(default)] chevron: Chevron,
    #[props(default)] span: TileSpan,
    onclick: EventHandler<Press>,
    #[props(default)] on_detail: Option<EventHandler<Press>>,
    #[props(default)] expanded: Expanded,
    #[props(default)] availability: Availability,
    #[props(default)] first: FirstShow,
    #[props(default)] disc: DiscMotion,
) -> Element {
    let live = availability == Availability::Enabled;
    let armed = use_armed();
    let listen = PressListeners::new(onclick);
    let detail = match chevron {
        Chevron::Detail => Some(chevron_button(&title, on_detail, expanded, availability)),
        Chevron::None => None,
    };
    rsx! {
        div {
            class: "ds-module-tile",
            role: "button",
            tabindex: "0",
            "data-span": span.slug(),
            "data-state": state.slug(),
            "aria-pressed": state.aria_pressed(),
            "aria-busy": state.aria_busy(),
            "aria-disabled": availability.aria_disabled(),
            onclick: move |event| {
                if live {
                    armed.arm(Touch::from_event(&event));
                    listen.click(&event);
                }
            },
            onkeydown: move |event| {
                if live && toggles(&event.key()) {
                    event.prevent_default();
                    armed.arm(Touch::from_event(&event));
                    onclick.call(Press::primary());
                }
            },
            ModuleDisc { glyph, state, motion: disc, first, armed }
            span { class: "ds-module-words",
                span { class: "ds-module-title", {text(&title)} }
                if let Some(status) = status {
                    span { class: "ds-module-status", {text(&status)} }
                }
            }
            {detail}
        }
    }
}

/// Whether a key on the tile toggles it: Enter or Space, as a button's own keys.
fn toggles(key: &Key) -> bool {
    matches!(key, Key::Enter) || *key == Key::Character(" ".into())
}

/// Whether a key on the chevron opens the detail: Enter, or Right (into the pane).
fn opens(key: &Key) -> bool {
    matches!(key, Key::Enter | Key::ArrowRight)
}

/// The chevron: its own hit target, named for the module, keeping every press it takes.
fn chevron_button(
    title: &TextLine,
    on_detail: Option<EventHandler<Press>>,
    expanded: Expanded,
    availability: Availability,
) -> Element {
    let name = format!("{} details", title.plain_text());
    let open = on_detail.filter(|_| availability == Availability::Enabled);
    let inert = open.map_or(Availability::Disabled, |_| Availability::Enabled);
    let listen = open.map(|open| PressListeners::new(open).with_propagation(Propagation::Stop));
    rsx! {
        button {
            r#type: "button",
            class: "ds-module-chevron",
            "aria-label": "{name}",
            title: "{name}",
            "aria-expanded": expanded.aria(),
            "aria-disabled": inert.aria_disabled(),
            onclick: move |event| match listen {
                Some(listen) => listen.click(&event),
                // An inert chevron still is not the tile: its press toggles nothing.
                None => {
                    event.stop_propagation();
                    kept_click(&event);
                }
            },
            onkeydown: move |event| {
                if opens(&event.key()) {
                    event.stop_propagation();
                    event.prevent_default();
                    if let Some(open) = open {
                        open.call(Press::primary());
                    }
                }
            },
            Glyph { icon: Icon::ChevronRight, size: IconSize::Compact }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{opens, toggles};
    use dioxus::prelude::Key;

    #[test]
    fn the_tile_and_its_chevron_take_their_own_keys() {
        let cases = [
            (Key::Enter, true, true),
            (Key::Character(" ".into()), true, false),
            (Key::ArrowRight, false, true),
            (Key::ArrowLeft, false, false),
            (Key::Tab, false, false),
        ];
        for (key, tile, chevron) in cases {
            assert_eq!(toggles(&key), tile, "tile {key:?}");
            assert_eq!(opens(&key), chevron, "chevron {key:?}");
        }
    }
}
