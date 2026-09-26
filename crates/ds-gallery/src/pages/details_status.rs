//! Details, the status glyphs (design/26-DETAILS.md 5.1, wave D1): one cell per glyph, a live
//! specimen moved by its buttons over a strip of every state it draws, at the bar's 22 px.

use super::Section;
use super::details::{Cell, mini};
use dioxus::prelude::*;
use ds::detail::EventStamp;
use ds::{
    BatteryGlyph, BatteryPower, BatteryState, BluetoothGlyph, BluetoothState, Fraction, IconPx,
    IconSize, LowAt, VolumeGlyph, VolumeState, VolumeWaves, WifiBars, WifiGlyph, WifiReach,
    WifiState,
};

/// The live specimen's size: twice the bar's, so its layers read on the page.
const LIVE: IconSize = IconSize::Px(IconPx(44));

/// A still of one state with its words under it.
#[component]
fn Still(words: String, children: Element) -> Element {
    rsx! {
        span { class: "g-status-still",
            {children}
            span { class: "g-status-words", "{words}" }
        }
    }
}

fn joined(bars: WifiBars, reach: WifiReach) -> WifiState {
    WifiState::Joined { bars, reach }
}

fn battery(level: u16, power: BatteryPower) -> BatteryState {
    BatteryState {
        level: Fraction(level),
        power,
        low_at: LowAt::default(),
    }
}

/// The section.
#[component]
pub fn StatusSection() -> Element {
    rsx! {
        Section { title: "Status glyphs", note: "The bar's status items as layers (D1). Wi-Fi searches one layer at a time while joining (after PendingGrace, held dimmed at PendingCap), fills once to its bars on joining, cross-fades its arcs, grows a \"!\" badge with no internet, shakes once on a failed join and draws its slash on when off. The battery's fill sweeps a step at a time, the bolt and plug grow in, and the fill turns red at the low threshold (never while charging). Bluetooth breathes while connecting and seals as its dots arrive. Volume is LevelGlyph's waves and slash.",
            div { class: "g-detail-grid g-status-grid",
                WifiCell {}
                BatteryCell {}
                BluetoothCell {}
                VolumeCell {}
            }
        }
    }
}

#[component]
fn WifiCell() -> Element {
    let mut state = use_signal(|| joined(WifiBars::Three, WifiReach::Internet));
    let mut stamp = use_signal(|| 0u32);
    let mut next = move || {
        *stamp.write() += 1;
        EventStamp(stamp())
    };
    let weaker = move || match state() {
        WifiState::Joined { bars, reach } => {
            let bars = match bars {
                WifiBars::Three => WifiBars::Two,
                WifiBars::Two => WifiBars::One,
                WifiBars::One => WifiBars::Three,
            };
            joined(bars, reach)
        }
        other => other,
    };
    let reach = move || match state() {
        WifiState::Joined {
            bars,
            reach: WifiReach::Internet,
        } => joined(bars, WifiReach::NoInternet),
        WifiState::Joined {
            bars,
            reach: WifiReach::NoInternet,
        } => joined(bars, WifiReach::Internet),
        other => other,
    };
    let stills = [
        WifiState::Off,
        WifiState::Idle,
        joined(WifiBars::One, WifiReach::Internet),
        joined(WifiBars::Two, WifiReach::Internet),
        joined(WifiBars::Three, WifiReach::Internet),
        joined(WifiBars::Three, WifiReach::NoInternet),
    ];
    rsx! {
        Cell { name: "Wi-Fi", code: "WifiGlyph {{ state: WifiState }}",
            controls: rsx! {
                {mini("Join", move |_| { let s = next(); state.set(WifiState::Joining(s)) })}
                {mini("Joined", move |_| state.set(joined(WifiBars::Three, WifiReach::Internet)))}
                {mini("Weaker", move |_| state.set(weaker()))}
                {mini("No internet", move |_| state.set(reach()))}
                {mini("Fail", move |_| { let s = next(); state.set(WifiState::Failed(s)) })}
                {mini("Off", move |_| state.set(WifiState::Off))}
            },
            div { class: "g-detail",
                WifiGlyph { state: state(), size: LIVE }
                span { class: "g-detail-word", {state().words()} }
            }
            div { class: "g-status-strip",
                for (index, still) in stills.into_iter().enumerate() {
                    Still { key: "{index}", words: still.words(), WifiGlyph { state: still } }
                }
            }
        }
    }
}

#[component]
fn BatteryCell() -> Element {
    let mut state = use_signal(|| battery(800, BatteryPower::Battery));
    let with_power = move |power| BatteryState { power, ..state() };
    let at = move |level| BatteryState {
        level: Fraction(level),
        ..state()
    };
    let stills = [
        battery(1000, BatteryPower::Battery),
        battery(550, BatteryPower::Battery),
        battery(150, BatteryPower::Battery),
        battery(600, BatteryPower::Charging),
        battery(1000, BatteryPower::Held),
    ];
    rsx! {
        Cell { name: "Battery", code: "BatteryGlyph {{ state: BatteryState }}",
            controls: rsx! {
                {mini("80 %", move |_| state.set(at(800)))}
                {mini("35 %", move |_| state.set(at(350)))}
                {mini("12 %", move |_| state.set(at(120)))}
                {mini("Charge", move |_| state.set(with_power(BatteryPower::Charging)))}
                {mini("Hold", move |_| state.set(with_power(BatteryPower::Held)))}
                {mini("Unplug", move |_| state.set(with_power(BatteryPower::Battery)))}
            },
            div { class: "g-detail",
                BatteryGlyph { state: state(), size: LIVE }
                span { class: "g-detail-word", {state().words()} }
            }
            div { class: "g-status-strip",
                for (index, still) in stills.into_iter().enumerate() {
                    Still { key: "{index}", words: still.words(), BatteryGlyph { state: still } }
                }
            }
        }
    }
}

#[component]
fn BluetoothCell() -> Element {
    let mut state = use_signal(|| BluetoothState::On);
    let mut stamp = use_signal(|| 0u32);
    let mut next = move || {
        *stamp.write() += 1;
        EventStamp(stamp())
    };
    let stills = [
        BluetoothState::Off,
        BluetoothState::On,
        BluetoothState::Connected,
    ];
    rsx! {
        Cell { name: "Bluetooth", code: "BluetoothGlyph {{ state: BluetoothState }}",
            controls: rsx! {
                {mini("Connect", move |_| { let s = next(); state.set(BluetoothState::Connecting(s)) })}
                {mini("Connected", move |_| state.set(BluetoothState::Connected))}
                {mini("Fail", move |_| { let s = next(); state.set(BluetoothState::Failed(s)) })}
                {mini("On", move |_| state.set(BluetoothState::On))}
                {mini("Off", move |_| state.set(BluetoothState::Off))}
            },
            div { class: "g-detail",
                BluetoothGlyph { state: state(), size: LIVE }
                span { class: "g-detail-word", {state().words()} }
            }
            div { class: "g-status-strip",
                for (index, still) in stills.into_iter().enumerate() {
                    Still { key: "{index}", words: still.words(), BluetoothGlyph { state: still } }
                }
            }
        }
    }
}

#[component]
fn VolumeCell() -> Element {
    let mut state = use_signal(|| VolumeState::Heard(VolumeWaves::Three));
    let quieter = move || match state() {
        VolumeState::Heard(VolumeWaves::Three) => VolumeState::Heard(VolumeWaves::Two),
        VolumeState::Heard(VolumeWaves::Two) => VolumeState::Heard(VolumeWaves::One),
        VolumeState::Heard(VolumeWaves::One) => VolumeState::Heard(VolumeWaves::Zero),
        VolumeState::Heard(VolumeWaves::Zero) | VolumeState::Muted | VolumeState::NoDevice => {
            VolumeState::Heard(VolumeWaves::Three)
        }
    };
    let stills = [
        VolumeState::Heard(VolumeWaves::Zero),
        VolumeState::Heard(VolumeWaves::One),
        VolumeState::Heard(VolumeWaves::Two),
        VolumeState::Heard(VolumeWaves::Three),
        VolumeState::Muted,
    ];
    rsx! {
        Cell { name: "Volume", code: "VolumeGlyph {{ state: VolumeState }}",
            controls: rsx! {
                {mini("Quieter", move |_| state.set(quieter()))}
                {mini("Mute", move |_| state.set(VolumeState::Muted))}
                {mini("No device", move |_| state.set(VolumeState::NoDevice))}
            },
            div { class: "g-detail",
                VolumeGlyph { state: state(), size: LIVE }
                span { class: "g-detail-word", {state().words()} }
            }
            div { class: "g-status-strip",
                for (index, still) in stills.into_iter().enumerate() {
                    Still { key: "{index}", words: still.words(), VolumeGlyph { state: still } }
                }
            }
        }
    }
}
