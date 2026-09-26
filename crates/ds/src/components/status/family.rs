//! StatusGlyph: the bar's status glyphs as one family, for a caller that holds any of them
//! (design/26-DETAILS.md section 7, wave D1).

use super::battery::BatteryGlyph;
use super::battery_state::BatteryState;
use super::bluetooth::BluetoothGlyph;
use super::bluetooth_state::BluetoothState;
use super::volume::{VolumeGlyph, VolumeState};
use super::wifi::WifiGlyph;
use super::wifi_state::WifiState;
use crate::icon::render::IconSize;
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
/// is a new glyph, mounted still.
#[component]
pub fn StatusGlyph(
    status: StatusState,
    #[props(default = IconSize::Bar)] size: IconSize,
) -> Element {
    match status {
        StatusState::Wifi(state) => rsx! { WifiGlyph { state, size } },
        StatusState::Battery(state) => rsx! { BatteryGlyph { state, size } },
        StatusState::Bluetooth(state) => rsx! { BluetoothGlyph { state, size } },
        StatusState::Volume(state) => rsx! { VolumeGlyph { state, size } },
    }
}
