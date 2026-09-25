//! Where a pointer lands on a horizontal track, shared by the form slider and the level control:
//! the one piece of their machines that is the same.

use crate::components::vocab::Fraction;
use crate::geometry::units::{Px, Rect};

/// The value under the pointer at `x` on a track occupying `rect`, clamped to its ends.
pub(crate) fn fraction_at(rect: Rect, x: Px) -> Fraction {
    let width = rect.size.width.0;
    if width <= 0.0 {
        return Fraction(0);
    }
    let share = ((x.0 - rect.left().0) / width).clamp(0.0, 1.0);
    // In 0..=1000 after the clamp, so the cast cannot truncate.
    Fraction((share * 1000.0).round() as u16)
}
