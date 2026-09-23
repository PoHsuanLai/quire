//! When an animation has finished, computed rather than observed: Blitz dispatches no
//! `animationend` (design/05-MOTION.md section 7.1).

use super::anim::Anim;
use crate::appearance::MotionLevel;
use crate::components::vocab::StaggerIndex;
use crate::time::FRAME_SLACK;
use crate::tokens::{DelayToken, ScalarToken, ScalarValue};
use std::time::Duration;

/// `duration(anim, level) + index x stagger(level) + FRAME_SLACK`.
///
/// Heal steps by `HealStep` (18 ms) rather than `--stagger`. Worked values, Post Standard:
/// `settle(Fold)` 454 ms, `settle(FoldHeavy)` 517 ms, `settle(Curl)` 594 ms; Reduced: every
/// settle is 94 ms.
pub fn settle(anim: Anim, level: MotionLevel, index: StaggerIndex) -> Duration {
    let duration = anim.recipe().duration.duration(level);
    duration + step(anim, level) * u32::from(index.get()) + FRAME_SLACK
}

/// The delay one index adds: the heal step for `Heal`, `--stagger` for everything else.
fn step(anim: Anim, level: MotionLevel) -> Duration {
    match (anim, ScalarToken::Stagger.value(level)) {
        (Anim::Heal, _) => DelayToken::HealStep.delay(level),
        (_, ScalarValue::Time(stagger)) => stagger,
        // `--stagger` is a time at every level; a table that said otherwise staggers nothing.
        (_, ScalarValue::Factor(_) | ScalarValue::Length(_) | ScalarValue::Angle(_)) => {
            Duration::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    use super::step;
    use crate::appearance::MotionLevel;
    use crate::motion::Anim;
    use std::time::Duration;

    #[test]
    fn each_index_adds_the_stagger_or_the_heal_step() {
        // `StaggerIndex::new` belongs to the component vocabulary; the per-index step is
        // checked here, the whole sum at index 0 in tests/motion_drift.rs. Worked values
        // (section 7.1): `settle(Rise, 12)` = 250 + 12 x 26 + 34 = 596 ms.
        const CASES: &[(Anim, MotionLevel, u64)] = &[
            (Anim::Rise, MotionLevel::Standard, 26),
            (Anim::Rise, MotionLevel::Extra, 34),
            (Anim::Rise, MotionLevel::Calm, 0),
            (Anim::Heal, MotionLevel::Standard, 18),
            (Anim::Heal, MotionLevel::Calm, 18),
            (Anim::Heal, MotionLevel::Reduced, 0),
            (Anim::PopIn, MotionLevel::Reduced, 0),
        ];
        for &(anim, level, ms) in CASES {
            assert_eq!(
                step(anim, level),
                Duration::from_millis(ms),
                "{anim:?} {level:?}"
            );
        }
    }
}
