//! The Widget looks page (design/23-WIDGETS.md): the widgets in "Neumorphism & Soft UI" for the
//! user's judgement, each widget in a Small and a Medium `WidgetFrame` on the desktop over a calm
//! wallpaper, in the page's scheme. The battery as a battery glyph and its percentage, one
//! device and four; the world clock as the time in the display face, and as four flat dials,
//! two by day and two by night.

use super::Section;
use crate::axes::Axes;
use crate::wallpaper;
use dioxus::prelude::*;
use ds::{
    Appearance, BatteryLevel, ClockFace, ClockLook, ClockTime, DayPhase, Ds, Fraction, Icon,
    Inject, Material, RingMark, RootChrome, Seconds, WidgetFrame, WidgetMetrics, WidgetSize,
    WidgetTitle, use_env,
};

/// One device on the battery widgets.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Device {
    name: &'static str,
    level: u16,
    mark: RingMark,
}

/// The medium widget's four: critical, half, full, and low but charging.
const DEVICES: [Device; 4] = [
    Device {
        name: "Mouse",
        level: 80,
        mark: RingMark::Plain,
    },
    Device {
        name: "Headphones",
        level: 450,
        mark: RingMark::Plain,
    },
    Device {
        name: "Keyboard",
        level: 1000,
        mark: RingMark::Plain,
    },
    Device {
        name: "This computer",
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
        second: Seconds::Hidden,
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
        time: ClockTime {
            hour: 3,
            minute: 9,
            second: Seconds::Shown(42),
        },
        phase: DayPhase::Night,
        offset: "-7 h",
    },
    City {
        name: "New York",
        time: at(22, 9),
        phase: DayPhase::Night,
        offset: "-12 h",
    },
    City {
        name: "Tokyo",
        time: at(11, 9),
        phase: DayPhase::Day,
        offset: "+1 h",
    },
];

/// The page.
#[component]
pub fn WidgetLooksPage() -> Element {
    rsx! {
        Section { title: "Battery", note: "BatteryLevel: an extruded battery body with an inset channel filled to the level (ink; amber at a fifth or less, red at a tenth, green while charging, with a bolt) and the percentage beside it. Small: this computer at 84 %, the percentage the hero. Medium: a row per device at 8, 45, 100 and 15 % charging.",
            Wall {
                BatterySmall {}
                BatteryMedium {}
            }
        }
        Section { title: "World clock", note: "ClockFace: Small, the time in the display face with the sun beside the city; Medium, four dials inset in the plate with extruded hands, Taipei and Tokyo by day (the plate's colour, ink hands), London (seconds shown) and New York by night (a near-black well, pale hands).",
            Wall {
                ClockSmall {}
                ClockMedium {}
            }
        }
    }
}

/// The wallpaper, with a desktop's Widget scope over it in the page's scheme.
#[component]
fn Wall(children: Element) -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (theme, accent, motion, blur) = {
        let axes = axes.read();
        (axes.theme, axes.accent, axes.motion, axes.blur)
    };
    let scheme = use_env().scheme;
    rsx! {
        div { class: "g-wall g-wl-wall", style: "background-image:url(\"{wallpaper::calm_uri(scheme)}\")",
            Ds {
                appearance: Appearance { theme, accent, motion },
                material: Material::Widget,
                blur,
                chrome: Some(RootChrome::Transparent),
                stylesheet: Inject::Host,
                div { class: "g-wl-cards", style: WidgetMetrics::default().style_attr(), {children} }
            }
        }
    }
}

fn percent(device: Device) -> u16 {
    (device.level + 5) / 10
}

#[component]
fn BatterySmall() -> Element {
    let title = Some(WidgetTitle::new(Icon::BatteryFull, "Battery"));
    rsx! {
        WidgetFrame { size: WidgetSize::Small, title,
            div { class: "g-wl-foot",
                div { class: "g-wl-hero",
                    span { class: "g-wl-number", "84" span { class: "g-wl-unit", "%" } }
                    BatteryLevel { level: Fraction(840), label: "This computer, 84%" }
                }
                span { class: "g-wl-label", "This computer" }
            }
        }
    }
}

#[component]
fn BatteryMedium() -> Element {
    let title = Some(WidgetTitle::new(Icon::BatteryFull, "Batteries"));
    rsx! {
        WidgetFrame { size: WidgetSize::Medium, title,
            div { class: "g-wl-rows",
                for device in DEVICES {
                    div { key: "{device.name}", class: "g-wl-row",
                        BatteryLevel { level: Fraction(device.level), mark: device.mark, label: device.name }
                        span { class: "g-wl-name", "{device.name}" }
                        span { class: "g-wl-pct", "{percent(device)}%" }
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
        WidgetFrame { size: WidgetSize::Small, title: Some(WidgetTitle::new(Icon::Clock, "Clock")),
            div { class: "g-wl-foot",
                ClockFace { time: city.time, phase: city.phase, look: ClockLook::Digital, label: city.name }
                span { class: "g-wl-label", "{city.offset}" }
            }
        }
    }
}

#[component]
fn ClockMedium() -> Element {
    rsx! {
        WidgetFrame { size: WidgetSize::Medium, title: Some(WidgetTitle::new(Icon::Clock, "World Clock")),
            div { class: "g-wl-dials",
                for city in CITIES {
                    div { key: "{city.name}", class: "g-wl-dial",
                        ClockFace { time: city.time, phase: city.phase, label: city.name }
                        span { class: "g-wl-label", "{city.offset}" }
                    }
                }
            }
        }
    }
}
