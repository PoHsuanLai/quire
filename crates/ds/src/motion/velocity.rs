//! How fast a person's hand was moving when it let go (design/27 section 3.12 rule 2): logical
//! pixels per second along the axis the motion runs, signed (right and down are positive). A
//! whole number, so [`crate::detail::Contact`] that carries one keeps `Eq` and `Hash`.

use super::swipe::Speed;

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
