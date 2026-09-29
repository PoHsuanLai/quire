//! `IdleDim`'s share: the pre-screen-off dim overlay's opacity, driven from Rust like [`super::
//! Sweep`] (design/26-DETAILS.md section 4.1, "Why Rust tweens") rather than a CSS keyframe,
//! because waking must snap the share to zero on input even mid-fade, and a CSS `animation`
//! cannot retarget without a restyle. sill's own idle service owns the phase this plays
//! (dim before screen-off, never real brightness; design/22-SETTINGS.md section 3.24
//! `idle.dim_s`/`idle.dim_level_pct`; sill FINDINGS "sill idle").
//!
//! Not built on [`super::Detailed`]/[`super::Cue`]: those classify an arbitrary state change into
//! one of the grammar's moments, but this primitive already knows exactly what a phase change
//! means (dim, or wake), so the extra layer would only translate one two-state enum into
//! another. [`plan`] plays the same role [`super::sweep::plan`] does for `Sweep`.

use super::glide::Glide;
use super::level::use_level;
use super::motor::use_motor;
use crate::core::vocab::{Fraction, Percent};
use crate::style::appearance::motion::MotionLevel;
use crate::style::tokens::{easing::EasingToken, timing::DurationToken};
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// What the caller wants: the full-brightness screen, or the overlay dimmed to its level. Two
/// states, not a `bool` (`CONVENTIONS.md#4-types`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdleDimPhase {
    /// Nothing drawn: full brightness.
    #[default]
    Awake,
    /// Dimmed to `level`.
    Dimmed,
}

/// What changed since the last render: a request the caller made, or a settings edit that moved
/// `level` while the phase stayed the same.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum IdleDimChange {
    /// `phase` itself changed.
    Phase,
    /// Only `level` changed, `phase` stayed `Dimmed`.
    Level,
}

/// Which way the share moves: fade in, or land on the target with no frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum IdleDimPlan {
    /// Fade to the target over `--t-idle-dim --e-out`.
    Fade,
    /// Stand at the target at once: waking always snaps (there is no exit to wait for), and a
    /// settings edit never animates (design/22-SETTINGS.md section 2, "no surface animates from
    /// a settings change"); Reduced never fades in either (R7: "no fade — straight to level").
    Snap,
}

/// Which plan `change` asks for, given the phase it is moving to and the level it plays at.
fn plan(change: IdleDimChange, phase: IdleDimPhase, level: MotionLevel) -> IdleDimPlan {
    match (change, phase, level) {
        (IdleDimChange::Level, ..) => IdleDimPlan::Snap,
        (IdleDimChange::Phase, IdleDimPhase::Awake, _) => IdleDimPlan::Snap,
        (IdleDimChange::Phase, IdleDimPhase::Dimmed, MotionLevel::Reduced) => IdleDimPlan::Snap,
        (
            IdleDimChange::Phase,
            IdleDimPhase::Dimmed,
            MotionLevel::Calm | MotionLevel::Standard | MotionLevel::Extra,
        ) => IdleDimPlan::Fade,
    }
}

/// The target share for `phase` at `level` (permille: `Percent(50)` is `500`).
fn target(phase: IdleDimPhase, level: Percent) -> i64 {
    match phase {
        IdleDimPhase::Awake => 0,
        IdleDimPhase::Dimmed => i64::from(u8::from(level)) * 10,
    }
}

/// The overlay's share now: `0` at [`IdleDimPhase::Awake`], `level` (in permille) at
/// [`IdleDimPhase::Dimmed`], fading in over `--t-idle-dim --e-out` and snapping the other way.
/// Retargets from where it is if asked again mid-fade (R10); asks for frames only while it moves
/// (R3).
pub fn use_idle_dim(level: Percent, phase: IdleDimPhase) -> Fraction {
    let env = use_level();
    let motor = use_motor(target(phase, level));
    let mut seen = use_hook(|| CopyValue::new((IdleDimPhase::Awake, level)));
    let now = (phase, level);
    if *seen.peek() != now {
        let (last_phase, _) = *seen.peek();
        let change = if last_phase == phase {
            IdleDimChange::Level
        } else {
            IdleDimChange::Phase
        };
        seen.set(now);
        queue_effect(move || {
            let motion = env.now();
            let to = target(phase, level);
            match plan(change, phase, motion) {
                IdleDimPlan::Snap => motor.snap(to),
                IdleDimPlan::Fade => motor.play(Glide {
                    from: motor.peek().value,
                    to,
                    length: DurationToken::IdleDim.duration(motion),
                    easing: EasingToken::Out.easing(motion),
                }),
            }
        });
    }
    let pose = motor.pose();
    Fraction(u16::try_from(pose.value.max(0)).unwrap_or(u16::MAX))
}

#[cfg(test)]
mod tests {
    use super::{IdleDimChange, IdleDimPhase, IdleDimPlan, plan, target};
    use crate::core::vocab::Percent;
    use crate::style::appearance::motion::MotionLevel;

    #[test]
    fn waking_always_snaps() {
        for level in [
            MotionLevel::Calm,
            MotionLevel::Standard,
            MotionLevel::Extra,
            MotionLevel::Reduced,
        ] {
            assert_eq!(
                plan(IdleDimChange::Phase, IdleDimPhase::Awake, level),
                IdleDimPlan::Snap,
                "{level:?}"
            );
        }
    }

    #[test]
    fn dimming_fades_except_under_reduced() {
        const CASES: &[(MotionLevel, IdleDimPlan)] = &[
            (MotionLevel::Calm, IdleDimPlan::Fade),
            (MotionLevel::Standard, IdleDimPlan::Fade),
            (MotionLevel::Extra, IdleDimPlan::Fade),
            (MotionLevel::Reduced, IdleDimPlan::Snap),
        ];
        for &(level, want) in CASES {
            assert_eq!(
                plan(IdleDimChange::Phase, IdleDimPhase::Dimmed, level),
                want,
                "{level:?}"
            );
        }
    }

    #[test]
    fn a_settings_edit_never_animates() {
        for phase in [IdleDimPhase::Awake, IdleDimPhase::Dimmed] {
            for level in [MotionLevel::Standard, MotionLevel::Reduced] {
                assert_eq!(
                    plan(IdleDimChange::Level, phase, level),
                    IdleDimPlan::Snap,
                    "{phase:?} {level:?}"
                );
            }
        }
    }

    #[test]
    fn the_target_is_zero_awake_and_the_level_in_permille_dimmed() {
        assert_eq!(target(IdleDimPhase::Awake, Percent(50)), 0);
        assert_eq!(target(IdleDimPhase::Dimmed, Percent(50)), 500);
        assert_eq!(target(IdleDimPhase::Dimmed, Percent(10)), 100);
        assert_eq!(target(IdleDimPhase::Dimmed, Percent(90)), 900);
    }
}
