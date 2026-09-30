//! Details, the status glyphs in their slots: the bar's four status items as
//! `MenuBarItem { image: Only }` holding a status glyph, at the bar's metrics.

use crate::pages::details::overview::{Cell, mini};
use dioxus::prelude::*;
use ds::components::content::status::battery_state::{BatteryPower, BatteryState, LowAt};
use ds::components::content::status::bluetooth_state::BluetoothState;
use ds::components::content::status::volume::{VolumeState, VolumeWaves};
use ds::components::content::status::wifi_state::{WifiBars, WifiReach, WifiState};
use ds::components::controls::button_model::ImagePosition;
use ds::prelude::*;
use ds_shell::prelude::*;
use ds_style::tokens::status::StatusMetrics;

/// One status item, labelled with its state's words.
#[component]
fn Item(status: StatusState) -> Element {
    rsx! {
        MenuBarItem {
            image: ImagePosition::Only,
            icon: status,
            label: status.words(),
            onclick: |_| {},
        }
    }
}

/// The cell.
#[component]
pub fn ItemsCell() -> Element {
    let mut battery = use_signal(|| BatteryState {
        level: Fraction(800),
        power: BatteryPower::Battery,
        low_at: LowAt::default(),
    });
    let at = move |level| BatteryState {
        level: Fraction(level),
        ..battery()
    };
    let with_power = move |power| BatteryState { power, ..battery() };
    let wifi = WifiState::Joined {
        bars: WifiBars::Three,
        reach: WifiReach::Internet,
    };
    rsx! {
        Cell { name: "In the bar", code: "MenuBarItem {{ image: Only, icon: StatusState }}",
            controls: rsx! {
                {mini("80 %", move |_| battery.set(at(800)))}
                {mini("15 %", move |_| battery.set(at(150)))}
                {mini("Charge", move |_| battery.set(with_power(BatteryPower::Charging)))}
                {mini("Unplug", move |_| battery.set(with_power(BatteryPower::Battery)))}
            },
            div { class: "g-status-strip", style: StatusMetrics::default().style_attr(),
                Item { status: StatusState::Wifi(wifi) }
                Item { status: StatusState::Bluetooth(BluetoothState::Connected) }
                Item { status: StatusState::Volume(VolumeState::Heard(VolumeWaves::Two)) }
                Item { status: StatusState::Battery(battery()) }
            }
        }
    }
}
