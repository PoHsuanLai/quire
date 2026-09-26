//! The Widget looks page (design/23-WIDGETS.md): the widgets drawn flat, bright and measured
//! (section 2), in the page's scheme over a calm wallpaper: the battery as rings with the device
//! glyph in them and the percentage under each, and the world clock as four flat dials with an
//! orange seconds hand; each once on the `Widget` material's plate and once with the Space's
//! tint on the card (`CardTint::Space`). The Widget reference page poses them as the reference
//! screenshots.

use super::Section;
use crate::axes::Axes;
use crate::wallpaper;
use dioxus::prelude::*;
use ds::{
    Appearance, BatteryFigure, BatteryLevel, CardTint, ClockFace, ClockTime, DayPhase, Ds,
    Fraction, Glyph, Icon, IconSize, Inject, Material, RingMark, RootChrome, Seconds, SpaceLook,
    WidgetFrame, WidgetMetrics, WidgetSize, use_env,
};

/// One device on the battery widgets.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Device {
    name: &'static str,
    icon: Icon,
    level: u16,
    mark: RingMark,
}

/// The medium widget's four: critical, half, full, and low but charging.
const DEVICES: [Device; 4] = [
    Device {
        name: "Mouse",
        icon: Icon::Mouse,
        level: 80,
        mark: RingMark::Plain,
    },
    Device {
        name: "Headphones",
        icon: Icon::Headphones,
        level: 450,
        mark: RingMark::Plain,
    },
    Device {
        name: "Keyboard",
        icon: Icon::Keyboard,
        level: 1000,
        mark: RingMark::Plain,
    },
    Device {
        name: "This computer",
        icon: Icon::Monitor,
        level: 150,
        mark: RingMark::Charging,
    },
];

/// A city on the clock widgets: its time, its phase and its offset from here.
#[derive(Debug, Clone, Copy, PartialEq)]
struct City {
    name: &'static str,
    time: ClockTime,
    phase: DayPhase,
    offset: &'static str,
}

const fn at(hour: u8, minute: u8) -> ClockTime {
    ClockTime {
        hour,
        minute,
        second: Seconds::Shown(30),
    }
}

const CITIES: [City; 4] = [
    City {
        name: "Taipei",
        time: at(10, 9),
        phase: DayPhase::Day,
        offset: "Today",
    },
    City {
        name: "London",
        time: at(3, 9),
        phase: DayPhase::Night,
        offset: "-7HRS",
    },
    City {
        name: "New York",
        time: at(22, 9),
        phase: DayPhase::Night,
        offset: "-12HRS",
    },
    City {
        name: "Tokyo",
        time: at(11, 9),
        phase: DayPhase::Day,
        offset: "+1HRS",
    },
];

/// The page.
#[component]
pub fn WidgetLooksPage() -> Element {
    rsx! {
        Section { title: "Battery", note: "BatteryLevel: a bright ring on a track of the plate darkened, the device glyph in it, the percentage under it. Small: this computer at 84 %. Medium: 8 % (red), 45 %, 100 % and 15 % charging (the bolt in the ring's gap). The second medium card carries the Space's tint.",
            Wall {
                BatterySmall {}
                BatteryMedium { tint: CardTint::Material }
                BatteryMedium { tint: CardTint::Space }
            }
        }
        Section { title: "World clock", note: "ClockFace: Small, one large dial with sixty ticks; Medium, four dials, Taipei and Tokyo by day (white), London and New York by night (dark), each with an orange seconds hand. The second medium card carries the Space's tint.",
            Wall {
                ClockSmall {}
                ClockMedium { tint: CardTint::Material }
                ClockMedium { tint: CardTint::Space }
            }
        }
    }
}

/// The wallpaper, with a desktop's Widget scope over it in the page's scheme.
#[component]
pub(super) fn Wall(children: Element) -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (theme, accent, motion, blur, look) = {
        let axes = axes.read();
        (
            axes.theme,
            axes.accent,
            axes.motion,
            axes.blur,
            axes.look.clone(),
        )
    };
    let scheme = use_env().scheme;
    rsx! {
        div { class: "g-wall g-wl-wall", style: "background-image:url(\"{wallpaper::calm_uri(scheme)}\")",
            Ds {
                appearance: Appearance { theme, accent, motion },
                look: SpaceLook { theme, ..look },
                material: Material::Widget,
                blur,
                chrome: Some(RootChrome::Transparent),
                stylesheet: Inject::Host,
                div { class: "g-wl-cards", style: WidgetMetrics::default().style_attr(), {children} }
            }
        }
    }
}

fn ring(device: Device) -> Element {
    rsx! {
        BatteryLevel { level: Fraction(device.level), mark: device.mark, label: device.name,
            Glyph { icon: device.icon, size: IconSize::Base }
        }
    }
}

#[component]
fn BatterySmall() -> Element {
    let computer = Device {
        name: "This computer",
        icon: Icon::Monitor,
        level: 840,
        mark: RingMark::Plain,
    };
    rsx! {
        WidgetFrame { size: WidgetSize::Small,
            div { class: "g-wr-solo",
                {ring(computer)}
                span { class: "g-wr-hero", BatteryFigure { level: Fraction(computer.level) } }
            }
        }
    }
}

#[component]
fn BatteryMedium(tint: CardTint) -> Element {
    rsx! {
        WidgetFrame { size: WidgetSize::Medium, tint,
            div { class: "g-wr-row",
                for device in DEVICES {
                    div { key: "{device.name}", class: "g-wr-cell",
                        {ring(device)}
                        span { class: "g-wr-figure", BatteryFigure { level: Fraction(device.level) } }
                    }
                }
            }
        }
    }
}

#[component]
fn ClockSmall() -> Element {
    let [city, ..] = CITIES;
    rsx! {
        WidgetFrame { size: WidgetSize::Small,
            ClockFace { time: city.time, phase: city.phase, label: city.name }
        }
    }
}

#[component]
fn ClockMedium(tint: CardTint) -> Element {
    rsx! {
        WidgetFrame { size: WidgetSize::Medium, tint,
            div { class: "g-wr-dials",
                for city in CITIES {
                    div { key: "{city.name}", class: "g-wr-dial",
                        ClockFace { time: city.time, phase: city.phase, label: city.name }
                        span { class: "g-wr-offset", "{city.offset}" }
                    }
                }
            }
        }
    }
}
