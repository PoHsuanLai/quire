//! The demo states the Details page and its frames play, each with its own `Detailed` table
//! written as an exhaustive match (design/26-DETAILS.md section 4.2).

use ds::detail::{Detailed, EventStamp, Moment};

/// A battery's charge in whole percent.
#[derive(Debug, Clone, PartialEq)]
pub enum Charge {
    /// At this percentage.
    Level(u16),
}

impl Detailed for Charge {
    fn moment(from: &Self, to: &Self) -> Moment {
        match (from, to) {
            (Charge::Level(a), Charge::Level(b)) if a == b => Moment::Rest,
            (Charge::Level(_), Charge::Level(_)) => Moment::Change,
        }
    }
    fn first(state: &Self) -> Moment {
        match state {
            Charge::Level(_) => Moment::Appear,
        }
    }
}

/// A Wi-Fi item's state.
#[derive(Debug, Clone, PartialEq)]
pub enum Net {
    /// The radio is off.
    Off,
    /// Joining a network.
    Joining,
    /// Joined.
    Joined,
    /// The join failed.
    Failed(EventStamp),
}

impl Detailed for Net {
    fn moment(from: &Self, to: &Self) -> Moment {
        match (from, to) {
            (Net::Off | Net::Joining | Net::Joined | Net::Failed(_), Net::Joining) => {
                Moment::Pending
            }
            (Net::Joining, Net::Joined) => Moment::Success,
            (Net::Off | Net::Joined | Net::Failed(_), Net::Joined) => Moment::Change,
            (Net::Off | Net::Joining | Net::Joined | Net::Failed(_), Net::Failed(_)) => {
                Moment::Failure
            }
            (Net::Off | Net::Joining | Net::Joined | Net::Failed(_), Net::Off) => {
                Moment::Unavailable
            }
        }
    }
    fn first(state: &Self) -> Moment {
        match state {
            Net::Joining => Moment::Pending,
            Net::Off | Net::Joined | Net::Failed(_) => Moment::Rest,
        }
    }
}

/// A seal: open, or sealed by one event.
#[derive(Debug, Clone, PartialEq)]
pub enum Seal {
    /// Not sealed.
    Open,
    /// Sealed.
    Sealed(EventStamp),
}

impl Detailed for Seal {
    fn moment(from: &Self, to: &Self) -> Moment {
        match (from, to) {
            (Seal::Open | Seal::Sealed(_), Seal::Sealed(_)) => Moment::Success,
            (Seal::Open | Seal::Sealed(_), Seal::Open) => Moment::Change,
        }
    }
    fn first(state: &Self) -> Moment {
        match state {
            Seal::Open | Seal::Sealed(_) => Moment::Rest,
        }
    }
}

/// A bell: quiet, or asking for the person.
#[derive(Debug, Clone, PartialEq)]
pub enum Bell {
    /// Nothing asked.
    Quiet,
    /// Asking, once per stamp.
    Asking(EventStamp),
}

impl Detailed for Bell {
    fn moment(from: &Self, to: &Self) -> Moment {
        match (from, to) {
            (Bell::Quiet | Bell::Asking(_), Bell::Asking(_)) => Moment::Attention,
            (Bell::Quiet | Bell::Asking(_), Bell::Quiet) => Moment::Change,
        }
    }
    fn first(state: &Self) -> Moment {
        match state {
            Bell::Quiet | Bell::Asking(_) => Moment::Rest,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Bell, Charge, Net, Seal};
    use ds::detail::{EventStamp, Moment, moment_table};

    #[test]
    fn the_demo_tables() {
        moment_table(&[
            (Charge::Level(80), Charge::Level(35), Moment::Change),
            (Charge::Level(80), Charge::Level(80), Moment::Rest),
        ]);
        moment_table(&[
            (Net::Off, Net::Joining, Moment::Pending),
            (Net::Joining, Net::Joined, Moment::Success),
            (Net::Joined, Net::Failed(EventStamp(1)), Moment::Failure),
            (Net::Joined, Net::Off, Moment::Unavailable),
        ]);
        moment_table(&[(Seal::Open, Seal::Sealed(EventStamp(1)), Moment::Success)]);
        moment_table(&[(Bell::Quiet, Bell::Asking(EventStamp(1)), Moment::Attention)]);
    }
}
