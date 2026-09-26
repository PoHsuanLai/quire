//! The bar's status glyphs as layers (design/26-DETAILS.md section 5.1, wave D1): Wi-Fi, battery,
//! Bluetooth and volume, each drawn from stacked parts on the Lucide grid and each with its own
//! `Detailed` state, so every change plays the moment its table names and then paints 0 frames.

mod battery;
mod battery_state;
mod bluetooth;
mod bluetooth_state;
mod family;
mod part;
mod slash;
mod volume;
mod wifi;
mod wifi_state;

pub use battery::BatteryGlyph;
pub use battery_state::{BatteryPower, BatteryState, LowAt};
pub use bluetooth::BluetoothGlyph;
pub use bluetooth_state::BluetoothState;
pub use family::{StatusGlyph, StatusState};
pub use volume::{VolumeGlyph, VolumeState, VolumeWaves};
pub use wifi::WifiGlyph;
pub use wifi_state::{WifiBars, WifiReach, WifiState};
