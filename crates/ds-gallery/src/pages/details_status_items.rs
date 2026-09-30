//! Details, the status glyphs in their slots: the bar's four status items as
//! `Button { Status }` holding a status glyph, at the bar's metrics.

use super::details::{Cell, mini};
use dioxus::prelude::*;
use ds::ImagePosition;
use ds::{
    BatteryPower, BatteryState, BluetoothState, Fraction, LowAt, StatusMetrics, StatusState,
    VolumeState, VolumeWaves, WifiBars, WifiReach, WifiState,
};
use ds_shell::MenuBarItem;

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
        Cell { name: "In the bar", code: "Button {{ variant: Status, icon: StatusState }}",
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
