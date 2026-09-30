//! What a `Stepper` steps through: its range and step, the direction of a press, and whether it
//! draws the value in a field beside the pair (data and pure arithmetic only).

use ds_core::word::Word;

/// Which half of the pair, or which arrow key, asks for the change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum StepDirection {
    /// The upper half: the value goes up.
    Up,
    /// The lower half: the value goes down.
    Down,
}

/// Whether the stepper draws the value it changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum Readout {
    /// The pair alone: the value is shown somewhere else (a label, a date's segment).
    Bare,
    /// A text field beside the pair, bound to the value: typing sets it, the pair steps it.
    #[default]
    Field,
}

/// The values a stepper moves through: `min` to `max` in steps of `step`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StepRange {
    min: i32,
    max: i32,
    step: i32,
}

impl StepRange {
    /// The range from `min` to `max` in steps of `step`. A `max` under `min` is `min`, and a
    /// step under 1 is 1, so every range has a value and a way to move.
    pub fn new(min: i32, max: i32, step: i32) -> Self {
        StepRange {
            min,
            max: max.max(min),
            step: step.max(1),
        }
    }

    /// The lowest value.
    pub fn min(self) -> i32 {
        self.min
    }

    /// The highest value.
    pub fn max(self) -> i32 {
        self.max
    }

    /// `value` brought into the range.
    pub fn clamp(self, value: i32) -> i32 {
        value.clamp(self.min, self.max)
    }

    /// The value one step from `value` toward `direction`, held at the ends.
    pub fn step_from(self, value: i32, direction: StepDirection) -> i32 {
        let delta = match direction {
            StepDirection::Up => self.step,
            StepDirection::Down => -self.step,
        };
        self.clamp(self.clamp(value).saturating_add(delta))
    }

    /// Whether `value` can still move toward `direction`.
    pub fn can_step(self, value: i32, direction: StepDirection) -> bool {
        self.step_from(value, direction) != self.clamp(value)
    }

    /// The value the text `typed` names, brought into the range; nothing for text that is not a
    /// whole number.
    pub fn parse(self, typed: &str) -> Option<i32> {
        typed
            .trim()
            .parse::<i32>()
            .ok()
            .map(|value| self.clamp(value))
    }
}

#[cfg(test)]
mod tests {
    use super::{StepDirection, StepRange};

    #[test]
    fn a_step_moves_one_step_and_stops_at_the_ends() {
        const CASES: &[(&str, i32, StepDirection, i32)] = &[
            ("up from the middle", 5, StepDirection::Up, 7),
            ("down from the middle", 5, StepDirection::Down, 3),
            ("up stops at the top", 9, StepDirection::Up, 10),
            ("down stops at the bottom", 1, StepDirection::Down, 0),
            (
                "a value above the range comes back in",
                40,
                StepDirection::Down,
                8,
            ),
        ];
        let range = StepRange::new(0, 10, 2);
        for &(name, from, direction, want) in CASES {
            assert_eq!(range.step_from(from, direction), want, "{name}");
        }
    }

    #[test]
    fn the_ends_cannot_step_further() {
        let range = StepRange::new(0, 10, 1);
        assert!(range.can_step(0, StepDirection::Up));
        assert!(!range.can_step(0, StepDirection::Down));
        assert!(!range.can_step(10, StepDirection::Up));
        assert!(range.can_step(10, StepDirection::Down));
    }

    #[test]
    fn typed_text_is_a_whole_number_in_range_or_nothing() {
        const CASES: &[(&str, Option<i32>)] = &[
            ("7", Some(7)),
            (" 7 ", Some(7)),
            ("-3", Some(0)),
            ("99", Some(10)),
            ("", None),
            ("-", None),
            ("7.5", None),
        ];
        let range = StepRange::new(0, 10, 1);
        for &(typed, want) in CASES {
            assert_eq!(range.parse(typed), want, "{typed:?}");
        }
    }

    #[test]
    fn a_degenerate_range_still_has_a_value_and_a_step() {
        let range = StepRange::new(5, 1, 0);
        assert_eq!((range.min(), range.max()), (5, 5));
        assert_eq!(range.step_from(5, StepDirection::Up), 5);
    }
}
