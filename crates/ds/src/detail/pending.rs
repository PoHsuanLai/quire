//! A bounded pending loop as data (design/26-DETAILS.md section 3.2, R4): which frame an
//! operation that has run for a given time shows, and when the next one is due.

use super::operation::Deadline;
use crate::appearance::MotionLevel;
use crate::tokens::{DelayToken, DurationToken};
use std::time::Duration;

/// How a pending loop moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PendingStyle {
    /// One layer at a time: the Wi-Fi bars searching.
    Iterate,
    /// Layers fill in turn and stay, then clear: a level being found.
    Cumulate,
    /// A disc's opacity between .45 and 1, one step per half.
    Breathe,
    /// A ring's dash turns a quarter per step.
    Spin,
}

/// How many layers a style steps through (the Wi-Fi glyph: the dot and three arcs = 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Layers(pub u8);

/// A pending loop's look.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PendingSpec {
    /// How it moves.
    pub style: PendingStyle,
    /// Over how many layers (1 for a ring or a disc).
    pub layers: Layers,
}

impl PendingSpec {
    /// Steps in one cycle: a layer each for Iterate, each fill and the clear for Cumulate, two
    /// halves for Breathe, four quarters for Spin.
    pub fn cycle(self) -> u8 {
        let layers = self.layers.0.max(1);
        match self.style {
            PendingStyle::Iterate => layers,
            PendingStyle::Cumulate => layers.saturating_add(1),
            PendingStyle::Breathe => 2,
            PendingStyle::Spin => 4,
        }
    }
}

/// What to draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PendingFrame {
    /// No operation, or one younger than `PendingGrace`: draw the state as it is.
    Idle,
    /// The loop's `n`th step since it showed (0 first). [`PendingFrame::lit`] reads a layer.
    Step(u8),
    /// Past the deadline, or under Reduced: the still frame (the glyph dimmed); 0 frames.
    Stalled,
}

/// Whether a layer is drawn in a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lit {
    /// Drawn.
    On,
    /// Not drawn (or, for Breathe, at its low half).
    Off,
}

impl PendingFrame {
    /// The `data-pending` word: `idle`, `step` or `still`.
    pub fn slug(self) -> &'static str {
        match self {
            PendingFrame::Idle => "idle",
            PendingFrame::Step(_) => "step",
            PendingFrame::Stalled => "still",
        }
    }

    /// Whether `layer` (0 first) is drawn in this frame of `spec`: Idle and Stalled draw every
    /// layer; Iterate lights one layer a step; Cumulate lights layers up to the step's and then
    /// none; Breathe is On on its high half; Spin is always On (it turns instead).
    pub fn lit(self, spec: PendingSpec, layer: u8) -> Lit {
        let PendingFrame::Step(n) = self else {
            return Lit::On;
        };
        let phase = n % spec.cycle();
        let on = match spec.style {
            PendingStyle::Iterate => layer == phase,
            PendingStyle::Cumulate => layer <= phase && phase < spec.layers.0,
            PendingStyle::Breathe => phase == 0,
            PendingStyle::Spin => true,
        };
        if on { Lit::On } else { Lit::Off }
    }
}

/// The frame an operation `elapsed` into its run shows at `level`, stopping at `deadline`.
pub(crate) fn frame_at(elapsed: Duration, deadline: Deadline, level: MotionLevel) -> PendingFrame {
    let grace = DelayToken::PendingGrace.delay(level);
    if elapsed < grace {
        return PendingFrame::Idle;
    }
    if level == MotionLevel::Reduced || elapsed >= deadline.length() {
        return PendingFrame::Stalled;
    }
    let step = DurationToken::PendingStep
        .duration(level)
        .as_millis()
        .max(1);
    let n = (elapsed - grace).as_millis() / step;
    PendingFrame::Step(u8::try_from(n).unwrap_or(u8::MAX))
}

/// How long until the frame after the one at `elapsed` is due, or `None` once the loop holds
/// still for good.
pub(crate) fn next_due(
    elapsed: Duration,
    deadline: Deadline,
    level: MotionLevel,
) -> Option<Duration> {
    let grace = DelayToken::PendingGrace.delay(level);
    if elapsed < grace {
        return Some(grace - elapsed);
    }
    if level == MotionLevel::Reduced || elapsed >= deadline.length() {
        return None;
    }
    let step = DurationToken::PendingStep.duration(level);
    let into = Duration::from_millis(
        u64::try_from((elapsed - grace).as_millis() % step.as_millis().max(1)).unwrap_or(0),
    );
    Some((step - into).min(deadline.length() - elapsed))
}

#[cfg(test)]
mod tests {
    use super::{Layers, Lit, PendingFrame, PendingSpec, PendingStyle, frame_at, next_due};
    use crate::appearance::MotionLevel;
    use crate::detail::Deadline;
    use std::time::Duration;

    const MS: fn(u64) -> Duration = Duration::from_millis;

    #[test]
    fn a_loop_waits_for_its_grace_steps_and_holds_at_its_deadline() {
        let cap = Deadline::cap();
        const CASES: &[(u64, MotionLevel, PendingFrame)] = &[
            (0, MotionLevel::Standard, PendingFrame::Idle),
            (399, MotionLevel::Standard, PendingFrame::Idle),
            (400, MotionLevel::Standard, PendingFrame::Step(0)),
            (699, MotionLevel::Standard, PendingFrame::Step(0)),
            (700, MotionLevel::Standard, PendingFrame::Step(1)),
            (9_999, MotionLevel::Standard, PendingFrame::Step(31)),
            (10_000, MotionLevel::Standard, PendingFrame::Stalled),
            (60_000, MotionLevel::Standard, PendingFrame::Stalled),
            (760, MotionLevel::Calm, PendingFrame::Step(1)),
            (399, MotionLevel::Reduced, PendingFrame::Idle),
            (400, MotionLevel::Reduced, PendingFrame::Stalled),
        ];
        for &(ms, level, want) in CASES {
            assert_eq!(frame_at(MS(ms), cap, level), want, "{ms} ms {level:?}");
        }
        assert_eq!(
            frame_at(
                MS(2_000),
                Deadline::within(MS(1_500)),
                MotionLevel::Standard
            ),
            PendingFrame::Stalled
        );
    }

    #[test]
    fn the_next_frame_is_due_at_the_next_step_and_never_after_the_deadline() {
        let cap = Deadline::cap();
        const CASES: &[(u64, MotionLevel, Option<u64>)] = &[
            (0, MotionLevel::Standard, Some(400)),
            (400, MotionLevel::Standard, Some(300)),
            (550, MotionLevel::Standard, Some(150)),
            (9_900, MotionLevel::Standard, Some(100)),
            (10_000, MotionLevel::Standard, None),
            (400, MotionLevel::Reduced, None),
        ];
        for &(ms, level, want) in CASES {
            assert_eq!(
                next_due(MS(ms), cap, level),
                want.map(MS),
                "{ms} ms {level:?}"
            );
        }
    }

    #[test]
    fn each_style_lights_its_own_layers() {
        let wifi = |style| PendingSpec {
            style,
            layers: Layers(4),
        };
        let lit =
            |frame: PendingFrame, spec| (0..4).map(|l| frame.lit(spec, l)).collect::<Vec<_>>();
        use Lit::{Off, On};
        assert_eq!(
            lit(PendingFrame::Step(2), wifi(PendingStyle::Iterate)),
            [Off, Off, On, Off]
        );
        assert_eq!(
            lit(PendingFrame::Step(5), wifi(PendingStyle::Iterate)),
            [Off, On, Off, Off]
        );
        assert_eq!(
            lit(PendingFrame::Step(1), wifi(PendingStyle::Cumulate)),
            [On, On, Off, Off]
        );
        assert_eq!(
            lit(PendingFrame::Step(4), wifi(PendingStyle::Cumulate)),
            [Off, Off, Off, Off]
        );
        assert_eq!(
            lit(PendingFrame::Stalled, wifi(PendingStyle::Iterate)),
            [On, On, On, On]
        );
        assert_eq!(
            lit(PendingFrame::Idle, wifi(PendingStyle::Cumulate)),
            [On, On, On, On]
        );
        let disc = PendingSpec {
            style: PendingStyle::Breathe,
            layers: Layers(1),
        };
        assert_eq!(PendingFrame::Step(0).lit(disc, 0), On);
        assert_eq!(PendingFrame::Step(1).lit(disc, 0), Off);
    }
}
