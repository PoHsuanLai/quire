//! The bar's status glyphs as layers (design/26-DETAILS.md section 5.1): Wi-Fi, battery,
//! Bluetooth and volume, each drawn from stacked parts on the Lucide grid and each with its own
//! `Detailed` state, so every change plays the moment its table names and then paints 0 frames.

pub(crate) mod battery;
pub mod battery_state;
pub(crate) mod bluetooth;
pub mod bluetooth_state;
pub(crate) mod family;
pub(crate) mod part;
pub(crate) mod slash;
pub(crate) mod stroked;
pub mod volume;
pub(crate) mod wifi;
pub mod wifi_state;
