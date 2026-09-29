//! Where playback is, and what each change of it means (design/26-DETAILS.md 5.2.10).

use crate::detail::{detailed::Detailed, moment::Moment, stamp::EventStamp};
use crate::icon::Icon;

/// Where the player is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Playback {
    /// Paused or stopped: the button offers Play; the position holds.
    #[default]
    Paused,
    /// Playing: the button offers Pause; the position advances a second at a time.
    Playing,
    /// Asked to play, waiting on data (the service's stamp for this wait): the art breathes after
    /// `PendingGrace`, held still at `PendingCap` (R4); the button already offers Pause.
    Buffering(EventStamp),
}

impl Playback {
    /// The glyph of the action the button offers next (Replace off-up: the next action, A5).
    pub(crate) fn next_action(self) -> Icon {
        match self {
            Playback::Paused => Icon::Play,
            Playback::Playing | Playback::Buffering(_) => Icon::Pause,
        }
    }

    /// The button's name: the action a press takes.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Playback::Paused => "Play",
            Playback::Playing | Playback::Buffering(_) => "Pause",
        }
    }

    /// Whether the position runs on its own between reports.
    pub(crate) fn clock(self) -> PositionClock {
        match self {
            Playback::Playing => PositionClock::Running,
            Playback::Paused | Playback::Buffering(_) => PositionClock::Held,
        }
    }
}

impl Detailed for Playback {
    fn moment(from: &Self, to: &Self) -> Moment {
        match (from, to) {
            (
                Playback::Paused | Playback::Playing | Playback::Buffering(_),
                Playback::Buffering(_),
            ) => Moment::Pending,
            (Playback::Paused, Playback::Paused) | (Playback::Playing, Playback::Playing) => {
                Moment::Rest
            }
            (Playback::Playing | Playback::Buffering(_), Playback::Paused)
            | (Playback::Paused | Playback::Buffering(_), Playback::Playing) => Moment::Change,
        }
    }

    /// A player shown buffering is a wait already running; otherwise it is simply there.
    fn first(state: &Self) -> Moment {
        match state {
            Playback::Buffering(_) => Moment::Pending,
            Playback::Paused | Playback::Playing => Moment::Rest,
        }
    }
}

/// Whether a position advances by itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum PositionClock {
    /// A second at a time from the last report.
    Running,
    /// Where the last report put it.
    Held,
}

#[cfg(test)]
mod tests {
    use super::Playback;
    use crate::detail::{
        detailed::{first_table, moment_table},
        moment::Moment,
        stamp::EventStamp,
    };
    use crate::icon::Icon;

    #[test]
    fn play_and_pause_are_changes_and_a_wait_is_pending() {
        let wait = Playback::Buffering(EventStamp(1));
        moment_table(&[
            (Playback::Paused, Playback::Playing, Moment::Change),
            (Playback::Playing, Playback::Paused, Moment::Change),
            (Playback::Paused, wait, Moment::Pending),
            (wait, Playback::Playing, Moment::Change),
            (wait, Playback::Buffering(EventStamp(2)), Moment::Pending),
            (Playback::Playing, Playback::Playing, Moment::Rest),
        ]);
        first_table(&[
            (wait, Moment::Pending),
            (Playback::Playing, Moment::Rest),
            (Playback::Paused, Moment::Rest),
        ]);
    }

    #[test]
    fn the_button_offers_the_next_action() {
        let cases = [
            (Playback::Paused, Icon::Play, "Play"),
            (Playback::Playing, Icon::Pause, "Pause"),
            (Playback::Buffering(EventStamp(1)), Icon::Pause, "Pause"),
        ];
        for (playback, icon, label) in cases {
            assert_eq!(
                (playback.next_action(), playback.label()),
                (icon, label),
                "{playback:?}"
            );
        }
    }
}
