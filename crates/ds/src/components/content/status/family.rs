//! StatusGlyph: the bar's status glyphs as one family, for a caller that holds any of them
//! (design/26-DETAILS.md section 7).

use super::battery::BatteryGlyph;
use super::battery_state::BatteryState;
use super::bluetooth::BluetoothGlyph;
use super::bluetooth_state::BluetoothState;
use super::volume::{VolumeGlyph, VolumeState};
use super::wifi::WifiGlyph;
use super::wifi_state::WifiState;
use crate::motion::detail::first_show::FirstShow;
use crate::style::icon::render::IconSize;
use dioxus::prelude::*;

/// One status item's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusState {
    /// The Wi-Fi item.
    Wifi(WifiState),
    /// The battery item.
    Battery(BatteryState),
    /// The Bluetooth item.
    Bluetooth(BluetoothState),
    /// The volume item.
    Volume(VolumeState),
}

impl StatusState {
    /// The state in words (R8).
    pub fn words(self) -> String {
        match self {
            StatusState::Wifi(state) => state.words().to_owned(),
            StatusState::Battery(state) => state.words(),
            StatusState::Bluetooth(state) => state.words().to_owned(),
            StatusState::Volume(state) => state.words().to_owned(),
        }
    }
}

/// The glyph for `status` at `size`. Each kind keeps its own moments; a status that changes kind
/// is a new glyph, mounted still. `first` reaches the glyphs that have an Appear (the battery's
/// fill sweeping in from empty): `Animate` on a surface the person just opened, `Still` (the
/// default) on bar chrome (R1).
#[component]
pub fn StatusGlyph(
    status: StatusState,
    #[props(default = IconSize::Bar)] size: IconSize,
    #[props(default)] first: FirstShow,
) -> Element {
    match status {
        StatusState::Wifi(state) => rsx! { WifiGlyph { state, size } },
        StatusState::Battery(state) => rsx! { BatteryGlyph { state, size, first } },
        StatusState::Bluetooth(state) => rsx! { BluetoothGlyph { state, size } },
        StatusState::Volume(state) => rsx! { VolumeGlyph { state, size } },
    }
}
