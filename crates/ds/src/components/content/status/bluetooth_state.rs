//! The Bluetooth item's state (design/26-DETAILS.md 5.1.2).

use ds_motion::detail::stamp::EventStamp;

/// What the Bluetooth item shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BluetoothState {
    /// The radio is off: the rune faint, slashed.
    Off,
    /// On, nothing connected: the rune.
    On,
    /// Connecting a device, the operation the service stamped.
    Connecting(EventStamp),
    /// At least one device connected: the rune between two dots (how many is the menu's).
    Connected,
    /// A connection failed, once per stamp.
    Failed(EventStamp),
}

impl BluetoothState {
    /// The state in words (R8).
    pub fn words(self) -> &'static str {
        match self {
            BluetoothState::Off => "Bluetooth off",
            BluetoothState::On => "Bluetooth on",
            BluetoothState::Connecting(_) => "Connecting…",
            BluetoothState::Connected => "Connected",
            BluetoothState::Failed(_) => "Couldn't connect",
        }
    }
}
