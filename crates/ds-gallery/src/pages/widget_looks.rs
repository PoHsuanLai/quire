//! The Widget looks page (design/23-WIDGETS.md): the candidate depth looks for the desktop
//! widgets, each in a Small and a Medium `WidgetFrame` on the desktop over the wallpaper, in the
//! page's scheme. The battery at three levels and charging, in each `BatteryLook`; the world
//! clock by day and by night, in each `DialLook`; then the card's two finishes side by side.
//! The first of each is what the widgets draw today, for comparison.

use super::Section;
use crate::axes::Axes;
use crate::wallpaper;
use dioxus::prelude::*;
use ds::{
    Appearance, BatteryLook, ClockFace, ClockLook, ClockTime, DayPhase, DialLook, Ds, Fraction,
    FrameFinish, Glyph, Icon, IconSize, Inject, LevelRing, Material, RingMark, RootChrome, Seconds,
    WidgetFrame, WidgetMetrics, WidgetSize, WidgetTitle,
};

/// One device on the battery widgets.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Device {
    name: &'static str,
    glyph: Option<Icon>,
    level: u16,
    mark: RingMark,
}

/// The medium widget's four: critical, half, full, and low but charging.
const DEVICES: [Device; 4] = [
    Device {
        name: "Mouse",
        glyph: Some(Icon::Mouse),
        level: 80,
        mark: RingMark::Plain,
    },
    Device {
        name: "Headphones",
        glyph: Some(Icon::Headphones),
        level: 450,
        mark: RingMark::Plain,
    },
    Device {
        name: "Keyboard",
        glyph: Some(Icon::Keyboard),
        level: 1000,
        mark: RingMark::Plain,
    },
    Device {
        name: "Laptop",
        glyph: Some(Icon::Monitor),
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
        time: at(9, 50),
        phase: DayPhase::Day,
        offset: "Today",
    },
    City {
        name: "London",
        time: ClockTime {
            hour: 2,
            minute: 50,
            second: Seconds::Shown(42),
        },
        phase: DayPhase::Night,
        offset: "-7 h",
    },
    City {
        name: "New York",
        time: at(21, 50),
        phase: DayPhase::Night,
        offset: "-12 h",
    },
    City {
        name: "Tokyo",
        time: at(10, 50),
        phase: DayPhase::Day,
        offset: "+1 h",
    },
];

/// What each look is, for its caption.
fn battery_note(look: BatteryLook) -> &'static str {
    match look {
        BatteryLook::Ring => "Ring (today): a stroked ring on a faint track, on the plain card",
        BatteryLook::Well => {
            "Well: the ring lies in a groove pressed into the plate, round a raised boss holding the glyph; the level is a glossy liquid; the bolt is a lit boss"
        }
        BatteryLook::Cell => {
            "Cell: a recessed capsule bed holding a glossy liquid as wide as the level, with a nub; low and critical recolour the liquid only"
        }
    }
}

fn dial_note(dial: DialLook) -> &'static str {
    match dial {
        DialLook::Paper => {
            "Paper (today): a flat paper disc by day, ink by night, on the plain card"
        }
        DialLook::Bezel => {
            "Bezel: a raised rim round a recessed sky face, a minute track, tapered hands over their shadow, a hub ringed in the accent"
        }
        DialLook::Sky => {
            "Sky: the dial is a well in the plate with the sky as its ground, dots and quarter bars, tapered hands over their shadow"
        }
    }
}

fn finish_note(finish: FrameFinish) -> &'static str {
    match finish {
        FrameFinish::Plain => "Plain (today): the Widget material's plate",
        FrameFinish::Lit => "Lit: the plate bevel, a sheen down the top and a shaded foot",
    }
}

/// The card finish each candidate is shown on: the current look on the plain card, the new
/// ones on the lit card.
fn finish_for(current: bool) -> FrameFinish {
    if current {
        FrameFinish::Plain
    } else {
        FrameFinish::Lit
    }
}

/// The page.
#[component]
pub fn WidgetLooksPage() -> Element {
    rsx! {
        Section { title: "Battery", note: "LevelRing {{ look }}: a Small card with this computer at 84 % (the hero percentage in the display face), and a Medium card with four devices: 8 % (critical), 45 %, 100 %, and 15 % charging (never low while charging).",
            Wall {
                for look in BatteryLook::ALL {
                    Candidate { note: battery_note(look),
                        BatterySmall { look }
                        BatteryMedium { look }
                    }
                }
            }
        }
        Section { title: "World clock", note: "ClockFace {{ dial }}: a Small card with the digital row and a Medium card with four analog dials, Taipei and Tokyo by day, London (seconds shown) and New York by night; the day dial is always the light pair and the night dial the dark pair, whatever the desktop.",
            Wall {
                for dial in DialLook::ALL {
                    Candidate { note: dial_note(dial),
                        ClockSmall { dial }
                        ClockMedium { dial }
                    }
                }
            }
        }
        Section { title: "Card finish", note: "WidgetFrame {{ finish }}: Plain is the Widget material's plate as it is; Lit adds the plate bevel of the app icons (a brighter top highlight, a sheen down the top, a shaded foot) inside the material's own edge and drop.",
            Wall {
                for finish in FrameFinish::ALL {
                    Candidate { note: finish_note(finish),
                        WidgetFrame { size: WidgetSize::Small, finish, title: Some(WidgetTitle::new(Icon::BatteryFull, "Battery")),
                            Hero { number: "84", label: "This computer",
                                LevelRing { level: Fraction(840), label: "This computer", look: BatteryLook::Cell }
                            }
                        }
                        ClockMediumIn { dial: DialLook::Sky, finish }
                    }
                }
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
    rsx! {
        div { class: "g-wall g-wl-wall", style: "background-image:url(\"{wallpaper::uri()}\")",
            Ds {
                appearance: Appearance { theme, accent, motion },
                material: Material::Widget,
                blur,
                chrome: Some(RootChrome::Transparent),
                stylesheet: Inject::Host,
                div { class: "g-wl-grid", style: WidgetMetrics::default().style_attr(), {children} }
            }
        }
    }
}

/// One candidate: its caption over its cards.
#[component]
fn Candidate(note: &'static str, children: Element) -> Element {
    rsx! {
        div { class: "g-wl-candidate",
            span { class: "g-wl-note", "{note}" }
            div { class: "g-wl-cards", {children} }
        }
    }
}

/// A hero value in the display face with its unit, the gauge under it, and a label.
#[component]
fn Hero(number: &'static str, label: &'static str, children: Element) -> Element {
    rsx! {
        div { class: "g-wl-hero",
            span { class: "g-wl-number", "{number}" span { class: "g-wl-unit", "%" } }
            {children}
            span { class: "g-wl-label", "{label}" }
        }
    }
}

#[component]
fn BatterySmall(look: BatteryLook) -> Element {
    let finish = finish_for(look == BatteryLook::Ring);
    let title = Some(WidgetTitle::new(Icon::BatteryFull, "Battery"));
    rsx! {
        WidgetFrame { size: WidgetSize::Small, finish, title,
            match look {
                BatteryLook::Ring => rsx! {
                    div { class: "g-wl-ring-small",
                        LevelRing { level: Fraction(840), label: "This computer", look }
                        span { class: "g-widgets-percent", "84%" }
                    }
                },
                BatteryLook::Well => rsx! {
                    div { class: "g-wl-well-small",
                        LevelRing { level: Fraction(840), label: "This computer", look,
                            Glyph { icon: Icon::Monitor, size: IconSize::Base }
                        }
                        div { class: "g-wl-stack",
                            span { class: "g-wl-number", "84" span { class: "g-wl-unit", "%" } }
                            span { class: "g-wl-label", "Built-in" }
                        }
                    }
                },
                BatteryLook::Cell => rsx! {
                    Hero { number: "84", label: "This computer",
                        LevelRing { level: Fraction(840), label: "This computer", look }
                    }
                },
            }
        }
    }
}

#[component]
fn BatteryMedium(look: BatteryLook) -> Element {
    let finish = finish_for(look == BatteryLook::Ring);
    let title = Some(WidgetTitle::new(Icon::BatteryFull, "Batteries"));
    let percent = |device: Device| (device.level + 5) / 10;
    rsx! {
        WidgetFrame { size: WidgetSize::Medium, finish, title,
            if look == BatteryLook::Cell {
                div { class: "g-wl-cells",
                    for device in DEVICES {
                        div { key: "{device.name}", class: "g-wl-cell-row",
                            div { class: "g-wl-cell-head",
                                if let Some(icon) = device.glyph {
                                    Glyph { icon, size: IconSize::Compact }
                                }
                                span { class: "g-wl-cell-name", "{device.name}" }
                                span { class: "g-wl-cell-pct", "{percent(device)}%" }
                            }
                            LevelRing { level: Fraction(device.level), mark: device.mark, label: device.name, look }
                        }
                    }
                }
            } else {
                div { class: "g-wl-rings",
                    for device in DEVICES {
                        div { key: "{device.name}", class: "g-wl-ring-col",
                            LevelRing { level: Fraction(device.level), mark: device.mark, label: device.name, look,
                                if let Some(icon) = device.glyph {
                                    Glyph { icon, size: IconSize::Base }
                                }
                            }
                            span { class: "g-wl-ring-pct", "{percent(device)}%" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ClockSmall(dial: DialLook) -> Element {
    let finish = finish_for(dial == DialLook::Paper);
    let [city, ..] = CITIES;
    rsx! {
        WidgetFrame { size: WidgetSize::Small, finish, title: Some(WidgetTitle::new(Icon::Clock, "Clock")),
            div { class: "g-wl-digital",
                ClockFace { time: city.time, phase: city.phase, look: ClockLook::Digital, label: city.name, dial }
                span { class: "g-wl-label", "{city.offset}" }
            }
        }
    }
}

#[component]
fn ClockMedium(dial: DialLook) -> Element {
    rsx! { ClockMediumIn { dial, finish: finish_for(dial == DialLook::Paper) } }
}

#[component]
fn ClockMediumIn(dial: DialLook, finish: FrameFinish) -> Element {
    rsx! {
        WidgetFrame { size: WidgetSize::Medium, finish, title: Some(WidgetTitle::new(Icon::Clock, "World Clock")),
            div { class: "g-wl-dials",
                for city in CITIES {
                    div { key: "{city.name}", class: "g-wl-stack g-wl-dial",
                        ClockFace { time: city.time, phase: city.phase, label: city.name, dial }
                        span { class: "g-wl-label", "{city.offset}" }
                    }
                }
            }
        }
    }
}
