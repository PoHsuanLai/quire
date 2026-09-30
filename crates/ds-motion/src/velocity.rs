//! How fast a person's hand was moving when it let go (design/27 section 3.12 rule 2): logical
//! pixels per second along the axis the motion runs, signed (right and down are positive). A
//! whole number, so [`crate::detail::touch::Contact`] that carries one keeps `Eq` and
//! `Hash`.

use super::swipe::Speed;
use ds_style::tokens::delay::DelayToken;

/// A release velocity in logical pixels per second.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Velocity(pub i32);

impl Velocity {
    /// Not moving: a tap, a key, a click.
    pub const ZERO: Velocity = Velocity(0);

    /// A swipe's measured speed, rounded to whole pixels per second.
    pub fn from_speed(speed: Speed) -> Velocity {
        let rounded = speed.0.round();
        Velocity(rounded.clamp(i32::MIN as f32, i32::MAX as f32) as i32)
    }

    /// As pixels per second.
    pub fn px_per_s(self) -> f64 {
        f64::from(self.0)
    }
}

/// One pointer position along a drag axis, and when.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Sample {
    at: ds_core::geometry::units::Px,
    when: std::time::Instant,
}

/// Measures a drag's release velocity from its last two moves (a drag's `onpointermove` feeds
/// it, its `onpointerup` reads it). Pure: the caller hands in the instants, from `ds::time::now`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct VelocityMeter {
    last: Option<Sample>,
    before: Option<Sample>,
}

impl VelocityMeter {
    /// The pointer is at `at` at `when`. A move in the same instant as the last (a host that
    /// repeats the position with the release) updates the last position and keeps the one
    /// before, so the speed is still measured over time that passed.
    pub fn moved(
        self,
        at: ds_core::geometry::units::Px,
        when: std::time::Instant,
    ) -> VelocityMeter {
        let same_instant = self.last.is_some_and(|last| last.when >= when);
        let before = if same_instant { self.before } else { self.last };
        VelocityMeter {
            last: Some(Sample { at, when }),
            before,
        }
    }

    /// The velocity at a release at `when`: nothing when the pointer had stopped (its last move
    /// was more than `DelayToken::ReleaseWindow` before) or moved only once.
    pub fn released(self, when: std::time::Instant) -> Velocity {
        let (Some(last), Some(before)) = (self.last, self.before) else {
            return Velocity::ZERO;
        };
        if when.saturating_duration_since(last.when) > DelayToken::ReleaseWindow.delay() {
            return Velocity::ZERO;
        }
        let seconds = last
            .when
            .saturating_duration_since(before.when)
            .as_secs_f32();
        if seconds <= 0.0 {
            return Velocity::ZERO;
        }
        Velocity::from_speed(Speed((last.at.0 - before.at.0) / seconds))
    }
}

#[cfg(test)]
mod tests {
    use super::{Velocity, VelocityMeter};
    use ds_core::geometry::units::Px;
    use std::time::{Duration, Instant};

    #[test]
    fn a_release_reads_the_last_two_moves_unless_the_pointer_stopped() {
        let t0 = Instant::now();
        let ms = |n: u64| t0 + Duration::from_millis(n);
        let meter = VelocityMeter::default()
            .moved(Px(0.0), ms(0))
            .moved(Px(10.0), ms(16))
            .moved(Px(30.0), ms(32));
        assert_eq!(meter.released(ms(40)), Velocity(1250));
        assert_eq!(
            meter.moved(Px(30.0), ms(32)).released(ms(40)),
            Velocity(1250),
            "a repeat"
        );
        assert_eq!(
            meter.released(ms(200)),
            Velocity::ZERO,
            "stopped before letting go"
        );
        assert_eq!(
            VelocityMeter::default()
                .moved(Px(5.0), ms(0))
                .released(ms(1)),
            Velocity::ZERO
        );
    }
}
