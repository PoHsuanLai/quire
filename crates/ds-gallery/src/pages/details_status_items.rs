//! Details, the status glyphs in their slots (sill Q390-Q392): the bar's four status items as
//! `IconButton { Status }` holding a status glyph, the battery item nudging once as it crosses
//! into low on battery (G11), at the bar's metrics.

use super::details::{Cell, mini};
use dioxus::prelude::*;
use ds::detail::{Detailed, FirstShow, Moment, Touch, use_detail};
use ds::{
    BatteryPower, BatteryState, BluetoothState, Fraction, IconButton, IconButtonVariant, LowAt,
    StatusMetrics, StatusState, VolumeState, VolumeWaves, WifiBars, WifiReach, WifiState,
};

/// The battery item's watch: crossing into low on battery is the one Attention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Watch {
    Clear,
    Low,
}

impl Detailed for Watch {
    fn moment(from: &Self, to: &Self) -> Moment {
        match (from, to) {
            (Watch::Clear, Watch::Low) => Moment::Attention,
            (Watch::Clear | Watch::Low, Watch::Clear) | (Watch::Low, Watch::Low) => Moment::Rest,
        }
    }

    fn first(state: &Self) -> Moment {
        match state {
            Watch::Clear | Watch::Low => Moment::Rest,
        }
    }
}

/// Whether `state` is low on battery.
fn watch(state: BatteryState) -> Watch {
    let LowAt(low) = state.low_at;
    match (state.power, state.level <= low) {
        (BatteryPower::Battery, true) => Watch::Low,
        (BatteryPower::Battery, false) | (BatteryPower::Charging | BatteryPower::Held, _) => {
            Watch::Clear
        }
    }
}

/// One status item, labelled with its state's words.
#[component]
fn Item(status: StatusState, nudge: Option<ds::detail::Cue>) -> Element {
    rsx! {
        IconButton {
            variant: IconButtonVariant::Status,
            icon: status,
            label: status.words(),
            onclick: |_| {},
            first: FirstShow::Still,
            nudge,
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
    let cue = use_detail(watch(battery()), FirstShow::Still, Touch::Remote).cue();
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
        Cell { name: "In the bar", code: "IconButton {{ variant: Status, icon: StatusState, nudge: Option<Cue> }}",
            controls: rsx! {
                {mini("80 %", move |_| battery.set(at(800)))}
                {mini("15 %", move |_| battery.set(at(150)))}
                {mini("Charge", move |_| battery.set(with_power(BatteryPower::Charging)))}
                {mini("Unplug", move |_| battery.set(with_power(BatteryPower::Battery)))}
            },
            div { class: "g-status-strip", style: StatusMetrics::default().style_attr(),
                Item { status: StatusState::Wifi(wifi), nudge: None }
                Item { status: StatusState::Bluetooth(BluetoothState::Connected), nudge: None }
                Item { status: StatusState::Volume(VolumeState::Heard(VolumeWaves::Two)), nudge: None }
                Item { status: StatusState::Battery(battery()), nudge: Some(cue) }
            }
        }
    }
}
