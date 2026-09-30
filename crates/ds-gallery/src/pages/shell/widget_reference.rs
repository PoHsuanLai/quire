//! The Widget reference page (design/23-WIDGETS.md section 1.1): our widgets posed as the
//! reference screenshots that section measures, at the same size (a 164 card, a 344 x 164
//! card), so a 2x snapshot sits beside a 2x reference crop pixel for pixel. The battery alone
//! (93 %), four small rings (two devices and two empty places), the medium row (83 %, 13 % low,
//! 99 % charging, 96 %), the small analog clock at 10:32:11, and the medium world clock at 4:25
//! in Cupertino, 8:25 in Tokyo, 9:25 in Sydney by day and 1:25 in Paris by night. Every card is
//! drawn through the widget contract (`WidgetCard` with quire's `BatteryWidget` and
//! `WorldClockWidget`, section 9), so the page proves the interface as well as the look.

use crate::pages::Section;
use crate::pages::shell::widget_blur::BlurWall;
use crate::pages::shell::widget_looks::Wall;
use dioxus::prelude::*;
use ds::components::content::status::battery_state::{BatteryPower, BatteryState, LowAt};
use ds::motion::wake::WakeStamp;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;
use ds_shell::battery::device_glyph::Device;
use ds_shell::clock::kind::{ClockTime, DayPhase, Seconds};
use ds_shell::prelude::*;
use ds_shell::widget::battery::{BatteryCell, BatteryEntry, BatteryWidget};
use ds_shell::widget::clock::{ClockCity, ClockEntry, WorldClockWidget};
use ds_shell::widget::timeline::Timeline;

/// One battery on the widgets.
pub(super) fn cell(name: &str, device: Device, level: u16, mark: BatteryPower) -> BatteryCell {
    BatteryCell {
        name: name.to_owned(),
        device,
        state: BatteryState {
            level: Fraction(level),
            power: mark,
            low_at: LowAt::default(),
        },
    }
}

/// The medium row, as the reference's: a phone at 83, a watch at 13 (low), earbuds charging at
/// 99, their case at 96 (our set draws the case as a speaker-like "other").
fn row() -> Vec<BatteryCell> {
    vec![
        cell("Phone", Device::Phone, 830, BatteryPower::Battery),
        cell("Watch", Device::Watch, 130, BatteryPower::Battery),
        cell("Earbuds", Device::Earbuds, 990, BatteryPower::Charging),
        cell("Case", Device::Other, 960, BatteryPower::Battery),
    ]
}

/// A city on the medium clock: its time, phase, day and offset.
fn city(name: &str, hour: u8, phase: DayPhase, day: &str, offset: &str) -> ClockCity {
    ClockCity {
        name: name.to_owned(),
        time: ClockTime {
            hour,
            minute: 25,
            second: Seconds::Shown(46),
        },
        phase,
        notes: vec![day.to_owned(), offset.to_owned()],
    }
}

fn cities() -> Vec<ClockCity> {
    vec![
        city("Cupertino", 16, DayPhase::Day, "Today", "-2HRS"),
        city("Tokyo", 8, DayPhase::Day, "Tomorrow", "+14HRS"),
        city("Sydney", 9, DayPhase::Day, "Tomorrow", "+15HRS"),
        city("Paris", 1, DayPhase::Night, "Tomorrow", "+7HRS"),
    ]
}

/// The page.
#[component]
pub fn WidgetReferencePage() -> Element {
    let mut wake = use_signal(WakeStamp::default);
    rsx! {
        Section { title: "Compositor blur", note: "The cards over a vivid wallpaper, blur on: behind each card a copy of the wallpaper blurred as the compositor blurs it (a Gaussian of sigma 22, the reference fit, design/23 M26), clipped to the card, so the tint composes over it as it will in the shell. The walls below paint what the toolbar's Blur axis asks for.",
            BlurWall {}
        }
        Section { title: "Batteries", note: "Small with one device (the ring at the top left, the percentage under it), small with four places, and medium with a row of four (13 % is low and red; 99 % is charging, with the bolt in the ring's gap). The device glyphs are the filled set (DeviceGlyph). Each ring fills from empty over --t-fill at --e-out as the page appears, its percentage counting alongside, and the bolt fades in when its ring has arrived (design/23 section 4.1); Replay passes the cards a new WakeStamp.",
            Button { common: Common { id: Some("replay-fill".to_string()), ..Common::default() }, size: ControlSize::Mini, label: "Replay the fill",
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

#[component]
pub(super) fn BatterySolo(#[props(default)] wake: WakeStamp) -> Element {
    let entry = BatteryEntry::Devices(vec![cell(
        "This computer",
        Device::Laptop,
        930,
        BatteryPower::Battery,
    )]);
    rsx! {
        WidgetCard { widget: BatteryWidget, timeline: Timeline::now(entry), size: WidgetSize::Small, wake }
    }
}

#[component]
pub(super) fn BatteryGrid(#[props(default)] wake: WakeStamp) -> Element {
    let entry = BatteryEntry::Devices(vec![
        cell("This computer", Device::Laptop, 930, BatteryPower::Battery),
        cell("Headphones", Device::Headphones, 800, BatteryPower::Battery),
    ]);
    rsx! {
        WidgetCard { widget: BatteryWidget, timeline: Timeline::now(entry), size: WidgetSize::Small, wake }
    }
}

#[component]
pub(super) fn BatteryRow(#[props(default)] wake: WakeStamp) -> Element {
    rsx! {
        WidgetCard { widget: BatteryWidget, timeline: Timeline::now(BatteryEntry::Devices(row())), size: WidgetSize::Medium, wake }
    }
}

#[component]
pub(super) fn ClockSmall() -> Element {
    let cupertino = ClockCity {
        name: "Cupertino".to_owned(),
        time: ClockTime {
            hour: 10,
            minute: 32,
            second: Seconds::Shown(11),
        },
        phase: DayPhase::Day,
        notes: Vec::new(),
    };
    rsx! {
        WidgetCard { widget: WorldClockWidget, timeline: Timeline::now(ClockEntry::Cities(vec![cupertino])), size: WidgetSize::Small }
    }
}

#[component]
pub(super) fn ClockMedium() -> Element {
    rsx! {
        WidgetCard { widget: WorldClockWidget, timeline: Timeline::now(ClockEntry::Cities(cities())), size: WidgetSize::Medium }
    }
}
