//! The battery item's state and what each change of it means (design/26-DETAILS.md 5.1.3, G8-G10).

use crate::components::vocab::Fraction;
use crate::detail::{Detailed, Moment};

/// Where the power comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatteryPower {
    /// On battery.
    #[default]
    Battery,
    /// Plugged in and charging: the bolt.
    Charging,
    /// Plugged in, not charging (full, or held by a charge limit): the plug.
    Held,
}

/// The level at or under which a discharging battery's fill turns `--battery-low` (R15). The
/// reference turns it at about a fifth; the shell reads it from `bar.battery_low_percent`
/// (proposed, design/26 G10), so it is the caller's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LowAt(pub Fraction);

impl Default for LowAt {
    /// A fifth.
    fn default() -> LowAt {
        LowAt(Fraction(200))
    }
}

/// What the battery item shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BatteryState {
    /// The charge, in thousandths.
    pub level: Fraction,
    /// Where the power comes from.
    pub power: BatteryPower,
    /// Where it turns low.
    pub low_at: LowAt,
}

/// Whether the fill states that the battery is low (charging is never low).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Tone {
    Normal,
    Low,
}

/// The fill's width in steps: the inner well is eleven grid units, drawn in half units, so one
/// step is under a logical pixel at the bar's 22 px and about one device pixel at 2x.
pub(crate) const STEPS: u16 = 22;

/// What the person can see of a battery state (R2): the fill's step, the mark, the tone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Look {
    pub(crate) step: u16,
    pub(crate) power: BatteryPower,
    pub(crate) tone: Tone,
}

impl BatteryState {
    /// What is drawn: the level quantised to the fill's steps, the mark, the tone.
    pub(crate) fn look(self) -> Look {
        let level = self.level.clamped().0;
        let tone = match (self.power, level <= self.low_at.0.0) {
            (BatteryPower::Battery, true) => Tone::Low,
            (BatteryPower::Battery, false) | (BatteryPower::Charging | BatteryPower::Held, _) => {
                Tone::Normal
            }
        };
        // Any charge at all shows at least one step.
        let step = match level {
            0 => 0,
            _ => ((level * STEPS + 500) / 1000).max(1),
        };
        Look {
            step,
            power: self.power,
            tone,
        }
    }

    /// The fill's share as drawn, in thousandths: the level on the fill's steps.
    pub(crate) fn drawn(self) -> Fraction {
        Fraction(self.look().step * 1000 / STEPS)
    }

    /// The state in words (R8), e.g. `80%, charging`.
    pub fn words(self) -> String {
        let percent = (self.level.clamped().0 + 5) / 10;
        let power = match self.power {
            BatteryPower::Battery => "on battery",
            BatteryPower::Charging => "charging",
            BatteryPower::Held => "plugged in",
        };
        format!("{percent}%, {power}")
    }
}

impl Detailed for BatteryState {
    fn moment(from: &Self, to: &Self) -> Moment {
        // A change to the level, the mark or the tone the person can see is a Change; anything
        // finer (80.4 then 80.1 %) is none (R2).
        if from.look() == to.look() {
            Moment::Rest
        } else {
            Moment::Change
        }
    }

    fn first(_: &Self) -> Moment {
        // The fill sweeps in from empty on a surface just opened; bar chrome passes
        // `FirstShow::Still`, which holds it back (R1).
        Moment::Appear
    }
}

#[cfg(test)]
mod tests {
    use super::{BatteryPower, BatteryState, LowAt, STEPS, Tone};
    use crate::components::vocab::Fraction;

    #[test]
    fn the_fill_is_drawn_in_steps_and_any_charge_shows() {
        let at = |level| BatteryState {
            level: Fraction(level),
            power: BatteryPower::Battery,
            low_at: LowAt::default(),
        };
        const CASES: &[(u16, u16)] = &[(0, 0), (5, 1), (500, 11), (1000, 22), (1400, 22)];
        for &(level, step) in CASES {
            assert_eq!(at(level).look().step, step, "{level}");
        }
        assert_eq!(at(1000).drawn(), Fraction(1000));
        assert_eq!(at(500).drawn(), Fraction(11 * 1000 / STEPS));
        assert_eq!(at(200).look().tone, Tone::Low);
        assert_eq!(at(201).look().tone, Tone::Normal);
    }
}
