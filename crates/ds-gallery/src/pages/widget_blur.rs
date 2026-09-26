//! The Widget reference page's "compositor blur" wall (design/23-WIDGETS.md section 1.1,
//! M26-M30): the widgets over a vivid wallpaper, each card over a copy of the wallpaper blurred
//! as the compositor blurs it and clipped to the card, so the card's `--m-tint` composes over it
//! exactly as it will in the shell. Blur is forced on here: the wall is the simulation of it.

use super::widget_reference::{BatteryGrid, BatteryRow, BatterySolo, ClockMedium, ClockSmall};
use crate::axes::Axes;
use crate::wallpaper_vivid::{self, HEIGHT, WIDTH};
use dioxus::prelude::*;
use ds::{Appearance, BlurState, Ds, Inject, Material, RootChrome, SpaceLook, WidgetMetrics};

/// Where a card sits on the wall, in logical pixels from its top left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Spot {
    left: u32,
    top: u32,
}

const fn spot(left: u32, top: u32) -> Spot {
    Spot { left, top }
}

/// The wall: the sharp wallpaper under a transparent desktop root with blur on.
#[component]
pub fn BlurWall() -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (theme, accent, motion, look) = {
        let axes = axes.read();
        (axes.theme, axes.accent, axes.motion, axes.look.clone())
    };
    let wall = format!(
        "width:{WIDTH}px;height:{HEIGHT}px;\
         background-image:url(\"{}\");background-size:{WIDTH}px {HEIGHT}px",
        wallpaper_vivid::sharp_uri()
    );
    let metrics = format!(
        "position:absolute;inset:0;{}",
        WidgetMetrics::default().style_attr()
    );
    rsx! {
        div { class: "g-wall g-wb-wall", style: "{wall}",
            Ds {
                appearance: Appearance { theme, accent, motion },
                look: SpaceLook { theme, ..look },
                material: Material::Widget,
                blur: BlurState::Available,
                chrome: Some(RootChrome::Transparent),
                stylesheet: Inject::Host,
                div { style: "{metrics}",
                    Slot { at: spot(24, 32), BatterySolo {} }
                    Slot { at: spot(208, 32), BatteryGrid {} }
                    Slot { at: spot(412, 32), BatteryRow {} }
                    Slot { at: spot(24, 232), ClockSmall {} }
                    Slot { at: spot(208, 232), ClockMedium {} }
                }
            }
        }
    }
}

/// One card at `at`, over the blurred wallpaper cut to the card's corner.
#[component]
fn Slot(at: Spot, children: Element) -> Element {
    let Spot { left, top } = at;
    let style = format!(
        "left:{left}px;top:{top}px;background-image:url(\"{}\");\
         background-size:{WIDTH}px {HEIGHT}px;background-position:-{left}px -{top}px",
        wallpaper_vivid::blurred_uri()
    );
    rsx! {
        div { class: "g-wb-slot", style: "{style}", {children} }
    }
}
