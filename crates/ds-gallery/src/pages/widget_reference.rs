//! The Widget reference page (design/23-WIDGETS.md section 1.1): our widgets posed as the
//! reference screenshots that section measures, at the same size (a 164 card, a 344 x 164
//! card), so a 2x snapshot sits beside a 2x reference crop pixel for pixel. The battery alone
//! (93 %), four small rings (two devices and two empty places), the medium row (83 %, 13 % low,
//! 99 % charging, 96 %), the small analog clock at 10:32:11, and the medium world clock at 4:25
//! in Cupertino, 8:25 in Tokyo, 9:25 in Sydney by day and 1:25 in Paris by night.

use super::Section;
use super::widget_blur::BlurWall;
use super::widget_looks::Wall;
use dioxus::prelude::*;
use ds::{
    BatteryFigure, BatteryLevel, Button, ButtonVariant, ClockFace, ClockTime, DayPhase, Fraction,
    Glyph, Icon, IconSize, RingMark, Seconds, WakeStamp, WidgetFrame, WidgetSize,
};

/// One device on the battery widgets.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Device {
    name: &'static str,
    icon: Icon,
    level: u16,
    mark: RingMark,
}

const fn device(name: &'static str, icon: Icon, level: u16, mark: RingMark) -> Device {
    Device {
        name,
        icon,
        level,
        mark,
    }
}

/// The medium row, as the reference's: a phone at 83, a watch at 13 (low), earbuds charging at
/// 99, their case at 96. Our set has no watch or case glyph, so a mouse and a keyboard stand in.
const ROW: [Device; 4] = [
    device("Phone", Icon::Phone, 830, RingMark::Plain),
    device("Mouse", Icon::Mouse, 130, RingMark::Plain),
    device("Headphones", Icon::Headphones, 990, RingMark::Charging),
    device("Keyboard", Icon::Keyboard, 960, RingMark::Plain),
];

/// A city on the medium clock: its time, phase, day and offset.
#[derive(Debug, Clone, Copy, PartialEq)]
struct City {
    name: &'static str,
    time: ClockTime,
    phase: DayPhase,
    day: &'static str,
    offset: &'static str,
}

const fn city(
    name: &'static str,
    hour: u8,
    phase: DayPhase,
    day: &'static str,
    offset: &'static str,
) -> City {
    City {
        name,
        time: ClockTime {
            hour,
            minute: 25,
            second: Seconds::Shown(46),
        },
        phase,
        day,
        offset,
    }
}

const CITIES: [City; 4] = [
    city("Cupertino", 16, DayPhase::Day, "Today", "-2HRS"),
    city("Tokyo", 8, DayPhase::Day, "Tomorrow", "+14HRS"),
    city("Sydney", 9, DayPhase::Day, "Tomorrow", "+15HRS"),
    city("Paris", 1, DayPhase::Night, "Tomorrow", "+7HRS"),
];

/// The page.
#[component]
pub fn WidgetReferencePage() -> Element {
    let mut wake = use_signal(WakeStamp::default);
    rsx! {
        Section { title: "Compositor blur", note: "The cards over a vivid wallpaper, blur on: behind each card a copy of the wallpaper blurred as the compositor blurs it (a Gaussian of sigma 22, the reference fit, design/23 M26), clipped to the card, so the tint composes over it as it will in the shell. The walls below paint what the toolbar's Blur axis asks for.",
            BlurWall {}
        }
        Section { title: "Batteries", note: "Small with one device (the ring at the top left, the percentage under it), small with four places, and medium with a row of four (13 % is low and red; 99 % is charging, with the bolt in the ring's gap). Each ring fills from empty over --t-fill at --e-out as the page appears, its percentage counting alongside, and the bolt fades in when its ring has arrived (design/23 section 4.1); Replay passes the rings a new WakeStamp.",
            Button { id: "replay-fill", variant: ButtonVariant::Mini, label: "Replay the fill",
                onclick: move |_| wake.set(wake().next()) }
            Wall {
                BatterySolo { wake: wake() }
                BatteryGrid { wake: wake() }
                BatteryRow { wake: wake() }
            }
        }
        Section { title: "Clock", note: "Small: one large day dial with sixty ticks and the seconds hand. Medium: four dials in a row, three by day and Paris by night, the city and the day and offset under each.",
            Wall {
                ClockSmall {}
                ClockMedium {}
            }
        }
    }
}

fn ring(device: Device, wake: WakeStamp) -> Element {
    rsx! {
        BatteryLevel { level: Fraction(device.level), mark: device.mark, label: device.name, wake,
            Glyph { icon: device.icon, size: IconSize::Base }
        }
    }
}

#[component]
pub(super) fn BatterySolo(#[props(default)] wake: WakeStamp) -> Element {
    rsx! {
        WidgetFrame { size: WidgetSize::Small,
            div { class: "g-wr-solo",
                {ring(device("This computer", Icon::Monitor, 930, RingMark::Plain), wake)}
                span { class: "g-wr-hero", BatteryFigure { level: Fraction(930), wake } }
            }
        }
    }
}

#[component]
pub(super) fn BatteryGrid(#[props(default)] wake: WakeStamp) -> Element {
    rsx! {
        WidgetFrame { size: WidgetSize::Small,
            div { class: "g-wr-grid",
                {ring(device("This computer", Icon::Monitor, 930, RingMark::Plain), wake)}
                {ring(device("Headphones", Icon::Headphones, 800, RingMark::Plain), wake)}
                BatteryLevel { level: Fraction(0), label: "No device" }
                BatteryLevel { level: Fraction(0), label: "No device" }
            }
        }
    }
}

#[component]
pub(super) fn BatteryRow(#[props(default)] wake: WakeStamp) -> Element {
    rsx! {
        WidgetFrame { size: WidgetSize::Medium,
            div { class: "g-wr-row",
                for device in ROW {
                    div { key: "{device.name}", class: "g-wr-cell",
                        {ring(device, wake)}
                        span { class: "g-wr-figure", BatteryFigure { level: Fraction(device.level), wake } }
                    }
                }
            }
        }
    }
}

#[component]
pub(super) fn ClockSmall() -> Element {
    let time = ClockTime {
        hour: 10,
        minute: 32,
        second: Seconds::Shown(11),
    };
    rsx! {
        WidgetFrame { size: WidgetSize::Small,
            ClockFace { time, label: "Cupertino" }
        }
    }
}

#[component]
pub(super) fn ClockMedium() -> Element {
    rsx! {
        WidgetFrame { size: WidgetSize::Medium,
            div { class: "g-wr-dials",
                for city in CITIES {
                    div { key: "{city.name}", class: "g-wr-dial",
                        ClockFace { time: city.time, phase: city.phase, label: city.name }
                        span { class: "g-wr-offset", "{city.day}" }
                        span { class: "g-wr-offset", "{city.offset}" }
                    }
                }
            }
        }
    }
}
