//! The rubber band (design/30-CATALOGUE.md section 1.3): past a limit a drag follows the pointer
//! by a share of the overshoot, and springs back on release. Swipe resistance and the host's
//! scroll physics read it; a slider does not stretch.

use crate::style::appearance::motion::MotionLevel;
use ds_core::geometry::units::Px;

/// The share of the overshoot the content follows (macOS's scroll rubber band, conf M).
pub const RUBBER_SHARE: f32 = 0.55;

/// How far the content is drawn past a limit when the pointer is `past` beyond it. Not past the
/// limit (zero or negative) it draws nothing extra. Under Reduced motion there is no band:
/// the limit is a hard stop.
pub fn resist(past: Px, level: MotionLevel) -> Px {
    match level {
        MotionLevel::Reduced => Px(0.0),
        MotionLevel::Standard => Px(past.0.max(0.0) * RUBBER_SHARE),
    }
}

#[cfg(test)]
mod tests {
    use super::resist;
    use crate::style::appearance::motion::MotionLevel::{Reduced, Standard};
    use ds_core::geometry::units::Px;

    #[test]
    fn the_band_follows_a_share_of_the_overshoot() {
        const CASES: &[(f32, f32)] = &[(-10.0, 0.0), (0.0, 0.0), (10.0, 5.5), (100.0, 55.0)];
        for &(past, want) in CASES {
            let drawn = resist(Px(past), Standard).0;
            assert!((drawn - want).abs() < 1e-4, "{past}: {drawn}");
        }
    }

    #[test]
    fn reduced_is_a_hard_stop() {
        assert_eq!(resist(Px(80.0), Reduced), Px(0.0));
    }
}
