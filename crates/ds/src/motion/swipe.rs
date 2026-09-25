//! Swipe to dismiss as a pure machine (sill Q122; design/13 section 13.3.6 "Swipe right to
//! dismiss"): a pointer drag moves the card 1:1 to the right and a quarter of the distance to
//! the left; released past the distance or the speed threshold it flies out to the right from
//! where it is, and under both it springs back. A horizontal scroll (a touchpad, a mouse's
//! tilt wheel) is summed into the same offset and decided the same way once the deltas stop,
//! since Blitz carries no scroll phase: the hook arms a quiet timer on each delta and feeds
//! [`SwipeInput::Quiet`] when it runs out.
//!
//! The machine decides; `use_swipe` owns the clock and the timers and draws the offset.

use crate::components::vocab::Fraction;
use crate::geometry::Px;
use std::time::Duration;

/// A speed in logical pixels per second.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Speed(pub f32);

/// When something happened, measured from any fixed origin the caller keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Stamp(pub Duration);

/// The swipe's thresholds (`notifications.swipe_dismiss_px`, `swipe_dismiss_velocity_px_s`,
/// `swipe_damping`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwipeMetrics {
    /// How far right a release must be to dismiss, 80.
    pub dismiss: Px,
    /// How fast rightwards a release must be moving to dismiss, 600 px/s.
    pub velocity: Speed,
    /// How much of a leftward drag the card follows, 250 (a quarter).
    pub damping: Fraction,
}

impl Default for SwipeMetrics {
    /// The keys' defaults (design/22 section 3.12).
    fn default() -> Self {
        SwipeMetrics {
            dismiss: Px(80.0),
            velocity: Speed(600.0),
            damping: Fraction(250),
        }
    }
}

/// How far a press may move and still be a press, not a drag (the pull tab's 3 px).
const TAP_SLOP: f32 = 3.0;

/// A move older than this at the release says the pointer had stopped: no fling.
const FLING_WINDOW: Duration = Duration::from_millis(100);

/// One pointer position along the swipe, and when.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Sample {
    x: Px,
    at: Stamp,
}

/// What moves the card.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Source {
    /// Nothing: the card rests at its offset (0, or springing back to it).
    Idle,
    /// A pointer that went down at `from`; the last two positions give the release speed.
    Pointer {
        from: Px,
        last: Sample,
        before: Option<Sample>,
    },
    /// Scroll deltas, summed into `raw` (undamped).
    Scroll { raw: Px },
    /// Released or scrolled past a threshold: flying out from the offset it had.
    Gone,
}

/// Whether the click that follows a pointer's release is a press on the card or the end of a
/// drag (swallowed, so a swipe never also opens the notification).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Click {
    /// The next click is a press.
    #[default]
    Passes,
    /// The next click ends a drag and is not a press.
    Swallowed,
}

/// Where a card is in a swipe: what moves it, how far it is off its place, and what the next
/// click means.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwipeState {
    source: Source,
    offset: Px,
    click: Click,
}

impl Default for SwipeState {
    fn default() -> Self {
        SwipeState {
            source: Source::Idle,
            offset: Px(0.0),
            click: Click::Passes,
        }
    }
}

/// What happened to a card.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SwipeInput {
    /// The pointer went down at `x`.
    Down { x: Px, at: Stamp },
    /// The pointer moved to `x` with the button down.
    Move { x: Px, at: Stamp },
    /// The pointer was released (or left the card, or a move came with no button down).
    Up { at: Stamp },
    /// A scroll delta: `dx` rightwards and `dy` downwards, in pixels.
    Scroll { dx: Px, dy: Px },
    /// No scroll delta has come for the quiet spell.
    Quiet,
}

/// What the hook does after a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwipeEffect {
    /// Nothing.
    None,
    /// (Re)start the quiet timer: a scroll delta was summed.
    ArmQuiet,
    /// The card is dismissed: it flies out to the right from its offset.
    Dismiss,
}

/// How the offset is drawn: following the input with no transition, or easing to where it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwipeLook {
    /// At rest or springing back to rest: the offset eases (`--t-move --e-spring`).
    Rest,
    /// Following a pointer or a scroll: no transition.
    Live,
    /// Dismissed: flying out.
    Gone,
}

impl SwipeLook {
    /// The `data-swipe` word.
    pub fn slug(self) -> &'static str {
        match self {
            SwipeLook::Rest => "rest",
            SwipeLook::Live => "live",
            SwipeLook::Gone => "gone",
        }
    }
}

impl SwipeState {
    /// How far right of its place the card is drawn (negative: left).
    pub fn offset(&self) -> Px {
        self.offset
    }

    /// How the offset is drawn.
    pub fn look(&self) -> SwipeLook {
        match self.source {
            Source::Idle => SwipeLook::Rest,
            Source::Pointer { .. } | Source::Scroll { .. } => SwipeLook::Live,
            Source::Gone => SwipeLook::Gone,
        }
    }

    /// What the next click means.
    pub fn click(&self) -> Click {
        self.click
    }

    /// A click was heard: the next one passes again.
    pub fn clicked(self) -> Self {
        SwipeState {
            click: Click::Passes,
            ..self
        }
    }

    /// One step.
    pub fn step(self, input: SwipeInput, metrics: SwipeMetrics) -> (Self, SwipeEffect) {
        let none = |state| (state, SwipeEffect::None);
        match (self.source, input) {
            (Source::Gone, _) => none(self),
            (Source::Idle | Source::Scroll { .. }, SwipeInput::Down { x, at }) => {
                none(SwipeState {
                    source: Source::Pointer {
                        from: x,
                        last: Sample { x, at },
                        before: None,
                    },
                    offset: Px(0.0),
                    click: Click::Passes,
                })
            }
            (Source::Pointer { from, last, .. }, SwipeInput::Move { x, at }) => {
                let raw = Px(x.0 - from.0);
                let moved = (raw.0.abs() >= TAP_SLOP) || self.click == Click::Swallowed;
                none(SwipeState {
                    source: Source::Pointer {
                        from,
                        last: Sample { x, at },
                        before: Some(last),
                    },
                    offset: shaped(raw, metrics),
                    click: if moved {
                        Click::Swallowed
                    } else {
                        Click::Passes
                    },
                })
            }
            (Source::Pointer { last, before, .. }, SwipeInput::Up { at }) => {
                let speed = release_speed(last, before, at);
                self.decide(speed, metrics)
            }
            (Source::Idle | Source::Scroll { .. }, SwipeInput::Scroll { dx, dy }) => {
                if dx.0.abs() <= dy.0.abs() {
                    return none(self);
                }
                let raw = match self.source {
                    Source::Scroll { raw } => Px(raw.0 + dx.0),
                    _ => dx,
                };
                (
                    SwipeState {
                        source: Source::Scroll { raw },
                        offset: shaped(raw, metrics),
                        click: self.click,
                    },
                    SwipeEffect::ArmQuiet,
                )
            }
            (Source::Scroll { .. }, SwipeInput::Quiet) => self.decide(Speed(0.0), metrics),
            (Source::Idle, SwipeInput::Move { .. } | SwipeInput::Up { .. } | SwipeInput::Quiet)
            | (Source::Scroll { .. }, SwipeInput::Move { .. } | SwipeInput::Up { .. })
            | (
                Source::Pointer { .. },
                SwipeInput::Down { .. } | SwipeInput::Scroll { .. } | SwipeInput::Quiet,
            ) => none(self),
        }
    }

    /// The end of a gesture moving at `speed`: past a threshold it is dismissed from where it
    /// is; under both it springs back to its place.
    fn decide(self, speed: Speed, metrics: SwipeMetrics) -> (Self, SwipeEffect) {
        let far = self.offset.0 >= metrics.dismiss.0;
        let fast = self.offset.0 > 0.0 && speed.0 >= metrics.velocity.0;
        if far || fast {
            let gone = SwipeState {
                source: Source::Gone,
                ..self
            };
            return (gone, SwipeEffect::Dismiss);
        }
        let back = SwipeState {
            source: Source::Idle,
            offset: Px(0.0),
            click: self.click,
        };
        (back, SwipeEffect::None)
    }
}

/// The offset a raw distance draws: all of it to the right, `damping` of it to the left.
fn shaped(raw: Px, metrics: SwipeMetrics) -> Px {
    if raw.0 >= 0.0 {
        raw
    } else {
        Px(raw.0 * f32::from(metrics.damping.clamped().0) / 1000.0)
    }
}

/// The pointer's speed at a release at `at`, from its last two positions; nothing when it had
/// stopped (its last move was more than [`FLING_WINDOW`] before the release) or moved once.
fn release_speed(last: Sample, before: Option<Sample>, at: Stamp) -> Speed {
    let Some(before) = before else {
        return Speed(0.0);
    };
    if at.0.saturating_sub(last.at.0) > FLING_WINDOW {
        return Speed(0.0);
    }
    let seconds = last.at.0.saturating_sub(before.at.0).as_secs_f32();
    if seconds <= 0.0 {
        return Speed(0.0);
    }
    Speed((last.x.0 - before.x.0) / seconds)
}

#[cfg(test)]
#[path = "swipe_tests.rs"]
mod tests;
