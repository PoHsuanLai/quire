//! The demo states the Details page and its frames play, each with its own `Detailed` table
//! written as an exhaustive match (design/26-DETAILS.md section 4.2).

use ds::detail::EventStamp;
use ds::prelude::*;

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

#[cfg(test)]
mod tests {
    use super::Net;
    use ds::detail::{EventStamp, moment_table};
    use ds::prelude::*;

    #[test]
    fn the_demo_tables() {
        moment_table(&[
            (Net::Off, Net::Joining, Moment::Pending),
            (Net::Joining, Net::Joined, Moment::Success),
            (Net::Joined, Net::Failed(EventStamp(1)), Moment::Failure),
            (Net::Joined, Net::Off, Moment::Unavailable),
        ]);
    }
}
