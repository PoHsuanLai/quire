//! Where a throw would come to rest if nothing caught it ([FLUID]; design/27 section 3.12 rule
//! 3): the scroll view's deceleration, `r = 0.998` of the speed kept per millisecond, projects a
//! release at `p` moving at `v` to `p + v·r/(1-r)` (in pixels per millisecond), about
//! `p + 0.5 s × v`. A thrown thing lands on the endpoint nearest that projection, not nearest
//! where the finger let go, so a flick carries it even from 30 % of the way.

use super::velocity::Velocity;
use ds_core::geometry::units::Px;

/// The deceleration rate per millisecond: 0.998, the scroll view's normal rate.
pub const DECELERATION_PER_MS: f64 = 0.998;

/// A release: where it let go, and how fast it was going.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Throw {
    /// Where the thing was when the hand let go.
    pub from: Px,
    /// How fast the hand was moving.
    pub velocity: Velocity,
}

impl Throw {
    /// Where the throw would come to rest: `from + v·r/(1-r)`.
    pub fn projected(self) -> Px {
        let per_ms = self.velocity.px_per_s() / 1000.0;
        let travel = per_ms * DECELERATION_PER_MS / (1.0 - DECELERATION_PER_MS);
        Px((f64::from(self.from.0) + travel) as f32)
    }

    /// The endpoint of `ends` nearest the projection; `None` when there are none.
    pub fn landing(self, ends: &[Px]) -> Option<Px> {
        let projected = self.projected().0;
        ends.iter()
            .copied()
            .min_by(|a, b| (a.0 - projected).abs().total_cmp(&(b.0 - projected).abs()))
    }
}

#[cfg(test)]
mod tests {
    use super::Throw;
    use crate::velocity::Velocity;
    use ds_core::geometry::units::Px;

    #[test]
    fn a_throw_projects_about_half_a_second_of_its_speed() {
        const CASES: &[(f32, i32, f32)] = &[
            (0.0, 0, 0.0),
            (0.0, 1000, 499.0),
            (100.0, -600, -199.4),
            (30.0, 1500, 778.5),
        ];
        for &(from, v, want) in CASES {
            let got = Throw {
                from: Px(from),
                velocity: Velocity(v),
            }
            .projected()
            .0;
            assert!((got - want).abs() < 0.01, "{from} at {v}: {got} != {want}");
        }
    }

    #[test]
    fn a_throw_lands_on_the_end_nearest_its_projection_not_its_release() {
        let ends = [Px(0.0), Px(100.0)];
        const CASES: &[(f32, i32, f32)] = &[
            // Released at 30 %: still, it falls back; flicked at 1500 px/s, it carries on.
            (30.0, 0, 0.0),
            (30.0, 1500, 100.0),
            // Released at 70 % but flicked back: it returns.
            (70.0, -400, 0.0),
            (70.0, 0, 100.0),
        ];
        for &(from, v, want) in CASES {
            let throw = Throw {
                from: Px(from),
                velocity: Velocity(v),
            };
            assert_eq!(throw.landing(&ends), Some(Px(want)), "{from} at {v}");
        }
        assert_eq!(
            Throw {
                from: Px(0.0),
                velocity: Velocity(0)
            }
            .landing(&[]),
            None
        );
    }
}
