//! The Bluetooth item's state and what each change of it means (design/26-DETAILS.md 5.1.2, G6,
//! G7).

use crate::detail::{Detailed, EventStamp, Moment};

/// What the Bluetooth item shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BluetoothState {
    /// The radio is off: the rune faint, slashed.
    Off,
    /// On, nothing connected: the rune.
    On,
    /// Connecting a device, the operation the service stamped: the rune breathes, bounded (R4).
    Connecting(EventStamp),
    /// At least one device connected: the rune between two dots (how many is the menu's).
    Connected,
    /// A connection failed, once per stamp: one shake, then the rune (R6).
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

impl Detailed for BluetoothState {
    fn moment(from: &Self, to: &Self) -> Moment {
        use BluetoothState::{Connected, Connecting, Failed, Off, On};
        match (from, to) {
            (a, b) if a == b => Moment::Rest,
            (Off | On | Connecting(_) | Connected | Failed(_), Connecting(_)) => Moment::Pending,
            (Connecting(_), Connected) => Moment::Success,
            (Off | On | Connected | Failed(_), Connected) => Moment::Change,
            (Off | On | Connecting(_) | Connected | Failed(_), Failed(_)) => Moment::Failure,
            (Off | On | Connecting(_) | Connected | Failed(_), Off) => Moment::Unavailable,
            (Off | On | Connecting(_) | Connected | Failed(_), On) => Moment::Change,
        }
    }

    fn first(state: &Self) -> Moment {
        match state {
            BluetoothState::Connecting(_) => Moment::Pending,
            BluetoothState::Off
            | BluetoothState::On
            | BluetoothState::Connected
            | BluetoothState::Failed(_) => Moment::Rest,
        }
    }
}
