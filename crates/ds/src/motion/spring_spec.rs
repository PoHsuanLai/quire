//! Which spring a motion runs on, decided by who moved it (design/27 section 3.12 rule 2;
//! [FLUID]): critical damping for a tap, a key or anything remote, 0.8 only when a hand let go
//! with momentum toward the target. [`SpringSpec::for_touch`] is the only way to a spring, so no
//! caller writes a raw damping.

use super::spring::{Millis, Ratio, SpringTuning};
use super::velocity::Velocity;
use crate::motion::detail::touch::Touch;
use crate::style::appearance::motion::MotionLevel;

/// A release slower than this, in pixels per second, carries no momentum: a press that barely
/// moved, not a flick.
const MOMENTUM_PX_PER_S: i32 = 50;

/// How quick a spring feels: its response (proposed, design/05 section 14.2; to be tuned beside
/// macOS).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SpringResponse {
    /// `--spring-quick` 300 ms: a knob, a selection indicator.
    Quick,
    /// `--spring-move` 450 ms: a panel, a sheet, a card returning.
    #[default]
    Move,
}

impl SpringResponse {
    /// The response.
    pub fn millis(self) -> Millis {
        match self {
            SpringResponse::Quick => Millis(300),
            SpringResponse::Move => Millis(450),
        }
    }

    /// The token's name.
    pub fn var(self) -> &'static str {
        match self {
            SpringResponse::Quick => "--spring-quick",
            SpringResponse::Move => "--spring-move",
        }
    }
}

/// A spring chosen by the touch that moved it.
///
/// ```
/// use ds::detail::Touch;
/// use ds::motion::{Ratio, SpringResponse, SpringSpec};
/// use ds::MotionLevel;
///
/// let remote = SpringSpec::for_touch(Touch::Remote).response(SpringResponse::Quick);
/// assert_eq!(remote.spring(1.0, MotionLevel::Standard).damping(), Ratio::CRITICAL);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpringSpec {
    touch: Touch,
    response: SpringResponse,
}

impl SpringSpec {
    /// The spring for a motion `touch` caused, at [`SpringResponse::Move`].
    pub fn for_touch(touch: Touch) -> SpringSpec {
        SpringSpec {
            touch,
            response: SpringResponse::Move,
        }
    }

    /// The same spring at another response.
    pub fn response(self, response: SpringResponse) -> SpringSpec {
        SpringSpec { response, ..self }
    }

    /// The touch it was chosen for.
    pub fn touch(self) -> Touch {
        self.touch
    }

    /// The velocity the hand hands to the spring, in pixels per second: the contact's release
    /// velocity; nothing for a remote change, and nothing under Reduced (no throw overshoots).
    pub fn thrown(self, level: MotionLevel) -> Velocity {
        match (level, self.touch) {
            (MotionLevel::Reduced, _) | (_, Touch::Remote) => Velocity::ZERO,
            (_, Touch::Contact(contact)) => contact.velocity(),
        }
    }

    /// The spring for a leg that has `travel` (target minus where it is, in pixels) to go:
    /// 0.8 when the release carried momentum toward the target, else critical; always critical
    /// under Reduced.
    pub fn spring(self, travel: f64, level: MotionLevel) -> SpringTuning {
        let v = self.thrown(level).0;
        let toward = v.abs() >= MOMENTUM_PX_PER_S && (f64::from(v) * travel) > 0.0;
        let damping = if toward {
            Ratio::MOMENTUM
        } else {
            Ratio::CRITICAL
        };
        SpringTuning::new(damping, self.response.millis())
    }
}

#[cfg(test)]
mod tests {
    use super::{SpringResponse, SpringSpec};
    use crate::motion::detail::touch::{Contact, Touch};
    use crate::motion::{
        spring::{Millis, Ratio},
        velocity::Velocity,
    };
    use crate::style::appearance::motion::MotionLevel;

    fn thrown(v: i32) -> Touch {
        Touch::Contact(Contact::for_tests().with_velocity(Velocity(v)))
    }

    #[test]
    fn damping_follows_the_touch_and_its_momentum() {
        let cases: &[(Touch, f64, MotionLevel, Ratio)] = &[
            (Touch::Remote, 100.0, MotionLevel::Standard, Ratio::CRITICAL),
            (thrown(0), 100.0, MotionLevel::Standard, Ratio::CRITICAL),
            (thrown(900), 100.0, MotionLevel::Standard, Ratio::MOMENTUM),
            (thrown(-900), 100.0, MotionLevel::Standard, Ratio::CRITICAL),
            (thrown(-900), -100.0, MotionLevel::Standard, Ratio::MOMENTUM),
            (thrown(30), 100.0, MotionLevel::Standard, Ratio::CRITICAL),
            (thrown(900), 100.0, MotionLevel::Reduced, Ratio::CRITICAL),
        ];
        for (touch, travel, level, want) in cases {
            let got = SpringSpec::for_touch(*touch)
                .spring(*travel, *level)
                .damping();
            assert_eq!(got, *want, "{touch:?} {travel} {level:?}");
        }
    }

    #[test]
    fn reduced_and_remote_hand_over_no_velocity() {
        assert_eq!(
            SpringSpec::for_touch(thrown(700)).thrown(MotionLevel::Standard),
            Velocity(700)
        );
        assert_eq!(
            SpringSpec::for_touch(thrown(700)).thrown(MotionLevel::Reduced),
            Velocity(0)
        );
        assert_eq!(
            SpringSpec::for_touch(Touch::Remote).thrown(MotionLevel::Standard),
            Velocity(0)
        );
    }

    #[test]
    fn responses_are_the_tokens() {
        let quick = SpringSpec::for_touch(Touch::Remote).response(SpringResponse::Quick);
        assert_eq!(
            quick.spring(0.0, MotionLevel::Standard).response(),
            Millis(300)
        );
        assert_eq!(
            SpringSpec::for_touch(Touch::Remote)
                .spring(0.0, MotionLevel::Standard)
                .response(),
            Millis(450)
        );
    }
}
