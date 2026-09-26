//! The Wi-Fi item's state and what each change of it means (design/26-DETAILS.md 5.1.1, G1-G5).

use crate::components::vocab::Fraction;
use crate::detail::{Detailed, EventStamp, Moment};

/// How many arcs a joined network lights, quantised from its strength (R2: 67 then 68 % is the
/// same three bars, no moment).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WifiBars {
    /// The smallest arc.
    One,
    /// Two arcs.
    Two,
    /// All three.
    Three,
}

impl WifiBars {
    /// The bars a strength (thousandths) shows: a third or less one, two thirds or less two,
    /// else three.
    pub fn of(strength: Fraction) -> WifiBars {
        match strength.0 {
            0..=333 => WifiBars::One,
            334..=666 => WifiBars::Two,
            _ => WifiBars::Three,
        }
    }

    /// The highest layer lit, from the dot (0) out.
    pub(crate) fn top(self) -> u8 {
        match self {
            WifiBars::One => 1,
            WifiBars::Two => 2,
            WifiBars::Three => 3,
        }
    }
}

/// Whether a joined network reaches the internet: the "!" badge says it does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WifiReach {
    /// Reaches it, or nobody knows yet (unknown is not a claim, R15).
    #[default]
    Internet,
    /// Joined, but no internet (or a portal or limited connectivity): the "!" badge.
    NoInternet,
}

/// What the Wi-Fi item shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WifiState {
    /// The radio is off: the fan faint, slashed.
    Off,
    /// On, joined to nothing: the fan faint.
    Idle,
    /// Joining (or searching for) a network, the operation the service stamped: the bounded
    /// searching loop. A new stamp is a new join and restarts the loop's grace and cap (R4).
    Joining(EventStamp),
    /// Joined: `bars` arcs lit, the rest faint, and the badge when there is no internet.
    Joined {
        /// The arcs lit.
        bars: WifiBars,
        /// Whether it reaches the internet.
        reach: WifiReach,
    },
    /// The join failed, once per stamp: one shake, then the fan faint (R6).
    Failed(EventStamp),
}

impl WifiState {
    /// The state in words, for the item's label and menu (R8: never motion alone).
    pub fn words(self) -> &'static str {
        match self {
            WifiState::Off => "Wi-Fi off",
            WifiState::Idle => "Not connected",
            WifiState::Joining(_) => "Joining…",
            WifiState::Joined {
                reach: WifiReach::Internet,
                ..
            } => "Connected",
            WifiState::Joined {
                reach: WifiReach::NoInternet,
                ..
            } => "No internet",
            WifiState::Failed(_) => "Couldn't join",
        }
    }
}

impl Detailed for WifiState {
    fn moment(from: &Self, to: &Self) -> Moment {
        use WifiState::{Failed, Idle, Joined, Joining, Off};
        match (from, to) {
            (a, b) if a == b => Moment::Rest,
            (Off | Idle | Joining(_) | Joined { .. } | Failed(_), Joining(_)) => Moment::Pending,
            (Joining(_), Joined { .. }) => Moment::Success,
            (Off | Idle | Joined { .. } | Failed(_), Joined { .. }) => Moment::Change,
            (Off | Idle | Joining(_) | Joined { .. } | Failed(_), Failed(_)) => Moment::Failure,
            (Off | Idle | Joining(_) | Joined { .. } | Failed(_), Off) => Moment::Unavailable,
            (Off | Idle | Joining(_) | Joined { .. } | Failed(_), Idle) => Moment::Change,
        }
    }

    fn first(state: &Self) -> Moment {
        match state {
            WifiState::Joining(_) => Moment::Pending,
            WifiState::Off | WifiState::Idle | WifiState::Joined { .. } | WifiState::Failed(_) => {
                Moment::Rest
            }
        }
    }
}
