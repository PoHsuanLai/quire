//! The Overlays page's widgets (sill Q182, Q183): over the wallpaper, the three sizes of
//! `WidgetFrame` on the desktop (the Widget material's card, light) and as tiles in the
//! notification center (a Popover panel, dark), holding the world clock in both looks and
//! phases and the battery `BatteryLevel` at three levels and charging. Live, a button drains the
//! battery a tenth at a time so its fill and percentage bump once each.

use super::Section;
use crate::axes::Axes;
use crate::wallpaper;
use dioxus::prelude::*;
use ds::{
    Appearance, BatteryFigure, BatteryLevel, Button, ButtonVariant, ClockFace, ClockLook,
    ClockTime, DayPhase, Ds, Fraction, Glyph, Icon, IconSize, Inject, Material, RingMark,
    RootChrome, Seconds, Theme, WidgetFrame, WidgetHost, WidgetMetrics, WidgetSize, WidgetTitle,
};

const TAIPEI: ClockTime = ClockTime {
    hour: 9,
    minute: 50,
    second: Seconds::Hidden,
};

const LONDON: ClockTime = ClockTime {
    hour: 2,
    minute: 50,
    second: Seconds::Shown(42),
};

/// The widgets section.
#[component]
pub fn Widgets() -> Element {
    rsx! {
        Section { title: "Widgets", note: "WidgetFrame on the grid unit WidgetMetrics writes (widgets.desktop_cell_px 164, desktop_gap_px 16): Small one cell, Medium 2x1, Large 2x2. Left, the desktop: the Widget material's card (its own 20 corner, padded 16) over the wallpaper, light. Right, the notification center: tiles on the Popover (--surface-2, a hairline, --r-tile 12, padded 12), dark. Inside: ClockFace analog (a flat pale dial by day, an ink dial by night, whatever the scheme; the second hand in --accent) and digital (the display face, bumping on each new minute, a sun or moon beside the city), and BatteryLevel at 8 % (--danger), 45 % and 100 % (--ink), and charging at 15 % (--ok and the bolt; never low while charging). Live: Drain lowers the battery a tenth, and its arc sweeps down from the old level as its percentage counts down with it.",
            div { class: "g-wall g-widgets", style: "background-image:url(\"{wallpaper::uri()}\")",
                Host { theme: Theme::Light, host: WidgetHost::Desktop }
                Host { theme: Theme::Dark, host: WidgetHost::Tile }
            }
        }
    }
}

/// The three sizes for `host`, in `theme`.
#[component]
fn Host(theme: Theme, host: WidgetHost) -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (accent, motion, blur) = {
        let axes = axes.read();
        (axes.accent, axes.motion, axes.blur)
    };
    let (material, chrome, class) = match host {
        WidgetHost::Desktop => (Material::Widget, RootChrome::Transparent, "g-widgets-desk"),
        WidgetHost::Tile => (Material::Popover, RootChrome::Painted, "g-widgets-center"),
    };
    rsx! {
        div { class,
            Ds {
                appearance: Appearance { theme, accent, motion },
                material,
                blur,
                chrome: Some(chrome),
                stylesheet: Inject::Host,
                div { class: "g-widgets-grid", style: WidgetMetrics::default().style_attr(),
                    Battery { host }
                    WidgetFrame { size: WidgetSize::Medium, host, title: Some(WidgetTitle::new(Icon::Clock, "World Clock")),
                        div { class: "g-widgets-clocks",
                            ClockFace { time: TAIPEI, label: "Taipei" }
                            ClockFace { time: LONDON, phase: DayPhase::Night, label: "London" }
                            ClockFace { time: TAIPEI, look: ClockLook::Digital, label: "Taipei" }
                            ClockFace { time: LONDON, phase: DayPhase::Night, look: ClockLook::Digital, label: "London" }
                        }
                    }
                    WidgetFrame { size: WidgetSize::Large, host, title: Some(WidgetTitle::new(Icon::BatteryFull, "Batteries")),
                        div { class: "g-widgets-batteries",
                            BatteryLevel { level: Fraction(80), label: "Mouse", Glyph { icon: Icon::Mouse, size: IconSize::Base } }
                            BatteryLevel { level: Fraction(450), label: "Headphones", Glyph { icon: Icon::Headphones, size: IconSize::Base } }
                            BatteryLevel { level: Fraction(1000), label: "Keyboard" }
                            BatteryLevel { level: Fraction(150), mark: RingMark::Charging, label: "This computer" }
                        }
                        div { class: "g-widgets-clocks",
                            ClockFace { time: TAIPEI, phase: DayPhase::Day, label: "Taipei" }
                            ClockFace { time: LONDON, phase: DayPhase::Night, label: "London" }
                        }
                    }
                }
            }
        }
    }
}

/// The small battery widget, drained a tenth at a time by its button.
#[component]
fn Battery(host: WidgetHost) -> Element {
    let mut level = use_signal(|| Fraction(840));
    rsx! {
        div { class: "g-col",
            WidgetFrame { size: WidgetSize::Small, host, title: Some(WidgetTitle::new(Icon::BatteryFull, "Battery")),
                div { class: "g-widgets-battery",
                    BatteryLevel { level: level(), label: "This computer" }
                    span { class: "g-widgets-percent", BatteryFigure { level: level() } }
                }
            }
            Button { variant: ButtonVariant::Mini, label: "Drain",
                onclick: move |_| level.set(Fraction(level().0.saturating_sub(100))) }
        }
    }
}
