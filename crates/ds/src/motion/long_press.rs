//! Long press (design/30-CATALOGUE.md section 1.4): a press held without moving becomes a
//! long press after `DelayToken::LongPress`. Pure: an event and the time in, the next state and
//! one effect out; the caller owns the timer. The dock's menu, the green button's menu and a
//! toolbar pull-down are long presses, and the state is the shared [`PressPhase`].

use ds_core::geometry::units::Point;
use ds_core::vocab::PressPhase;
use ds_style::tokens::delay::DelayToken;
use std::time::{Duration, Instant};

/// How far a press may move and still be held still, in pixels.
pub const LONG_PRESS_SLOP: f32 = 4.0;

/// Something that happened to the pressed control.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LongPressEvent {
    /// The button (or a key) went down at `at`.
    Down {
        /// Where.
        at: Point,
        /// When.
        now: Instant,
    },
    /// The pointer moved to `at` with the button down.
    Move {
        /// Where.
        at: Point,
    },
    /// The button came up.
    Up,
    /// The timer fired.
    Due {
        /// When.
        now: Instant,
    },
}

/// What the caller must do after a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LongPressEffect {
    /// Nothing.
    None,
    /// Start a timer that fires `Due` after `after`.
    Arm {
        /// The long-press delay.
        after: Duration,
    },
    /// Cancel the timer.
    Disarm,
    /// The press has become a long press: show the menu.
    Fire,
}

/// The machine: the phase, and where and when the press began.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LongPress {
    phase: PressPhase,
    from: Option<(Point, Instant)>,
}

impl LongPress {
    /// Where the press stands.
    pub fn phase(&self) -> PressPhase {
        self.phase
    }

    /// Apply `event`.
    pub fn step(self, event: LongPressEvent) -> (Self, LongPressEffect) {
        match (self.phase, event) {
            (PressPhase::Idle, LongPressEvent::Down { at, now }) => (
                LongPress {
                    phase: PressPhase::Pressed,
                    from: Some((at, now)),
                },
                LongPressEffect::Arm {
                    after: DelayToken::LongPress.delay(),
                },
            ),
            (PressPhase::Pressed, LongPressEvent::Move { at })
                if self
                    .from
                    .is_some_and(|(from, _)| travelled(from, at) > LONG_PRESS_SLOP) =>
            {
                (LongPress::default(), LongPressEffect::Disarm)
            }
            (PressPhase::Pressed, LongPressEvent::Up) => {
                (LongPress::default(), LongPressEffect::Disarm)
            }
            (PressPhase::Held, LongPressEvent::Up) => (LongPress::default(), LongPressEffect::None),
            (PressPhase::Pressed, LongPressEvent::Due { now })
                if self.from.is_some_and(|(_, since)| {
                    now.saturating_duration_since(since) >= DelayToken::LongPress.delay()
                }) =>
            {
                (
                    LongPress {
                        phase: PressPhase::Held,
                        ..self
                    },
                    LongPressEffect::Fire,
                )
            }
            _ => (self, LongPressEffect::None),
        }
    }
}

/// The straight-line distance between two points.
fn travelled(from: Point, to: Point) -> f32 {
    (to.x.0 - from.x.0).hypot(to.y.0 - from.y.0)
}

#[cfg(test)]
mod tests {
    use super::{LongPress, LongPressEffect as Effect, LongPressEvent as Event};
    use ds_core::geometry::units::{Point, Px};
    use ds_core::vocab::PressPhase::{Held, Idle, Pressed};
    use ds_style::tokens::delay::DelayToken;
    use std::time::{Duration, Instant};

    fn at(x: f32, y: f32) -> Point {
        Point { x: Px(x), y: Px(y) }
    }

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    /// Feed `events`, returning the final phase and every effect.
    fn run(events: &[Event]) -> (LongPress, Vec<Effect>) {
        events.iter().fold(
            (LongPress::default(), Vec::new()),
            |(state, mut seen), event| {
                let (next, effect) = state.step(*event);
                seen.push(effect);
                (next, seen)
            },
        )
    }

    #[test]
    fn a_press_held_still_fires_once_at_the_delay() {
        let t = Instant::now();
        let delay = DelayToken::LongPress.delay();
        let (state, seen) = run(&[
            Event::Down {
                at: at(10.0, 10.0),
                now: t,
            },
            Event::Move { at: at(12.0, 11.0) },
            Event::Due { now: t + delay },
        ]);
        assert_eq!(state.phase(), Held);
        assert_eq!(
            seen,
            [Effect::Arm { after: delay }, Effect::None, Effect::Fire]
        );
        let (state, effect) = state.step(Event::Due {
            now: t + delay + ms(50),
        });
        assert_eq!(
            (state.phase(), effect),
            (Held, Effect::None),
            "it fires once"
        );
    }

    #[test]
    fn moving_past_the_slop_cancels_it() {
        let t = Instant::now();
        let (state, seen) = run(&[
            Event::Down {
                at: at(0.0, 0.0),
                now: t,
            },
            Event::Move { at: at(5.0, 0.0) },
            Event::Due { now: t + ms(600) },
        ]);
        assert_eq!(state.phase(), Idle);
        assert_eq!(seen[1], Effect::Disarm);
        assert_eq!(seen[2], Effect::None, "the late timer is ignored");
    }

    #[test]
    fn a_release_before_the_delay_is_a_plain_press() {
        let t = Instant::now();
        let (state, seen) = run(&[
            Event::Down {
                at: at(0.0, 0.0),
                now: t,
            },
            Event::Up,
        ]);
        assert_eq!(state.phase(), Idle);
        assert_eq!(seen[1], Effect::Disarm);
    }

    #[test]
    fn a_stale_timer_before_the_delay_does_nothing() {
        let t = Instant::now();
        let (state, _) = run(&[
            Event::Down {
                at: at(0.0, 0.0),
                now: t,
            },
            Event::Due { now: t + ms(100) },
        ]);
        assert_eq!(state.phase(), Pressed);
    }

    #[test]
    fn a_held_press_survives_movement_and_ends_on_release() {
        let t = Instant::now();
        let delay = DelayToken::LongPress.delay();
        let (state, _) = run(&[
            Event::Down {
                at: at(0.0, 0.0),
                now: t,
            },
            Event::Due { now: t + delay },
            Event::Move { at: at(40.0, 40.0) },
        ]);
        assert_eq!(state.phase(), Held);
        let (state, effect) = state.step(Event::Up);
        assert_eq!((state.phase(), effect), (Idle, Effect::None));
    }
}
