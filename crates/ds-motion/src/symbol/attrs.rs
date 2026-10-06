//! What a symbol's wrapper carries for the stylesheet's share of an effect: the animation class,
//! the restart alias and whether it stands hidden. A pure step over the last render's memory, so
//! a table pins when an effect fires: on a change of its trigger, activity or visibility, never
//! on a render that changes nothing (design/35-SYMBOL-EFFECTS.md section 2).

use super::effect::{Activity, SymbolEffect, TransitionEffect};
use super::route::{loop_anim, once_anim};
use crate::anim::Anim;
use ds_core::vocab::Shown;

/// What the wrapper wears.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SymbolAttrs {
    /// The animation it plays, if any.
    pub anim: Option<Anim>,
    /// The keyframe alias (`data-pulse`), flipped on each fire so a repeat restarts.
    pub pulse: Option<&'static str>,
    /// Whether it stands invisible with nothing playing.
    pub hidden: bool,
}

impl SymbolAttrs {
    /// The animation's utility class (`a-bounce`).
    pub fn class(self) -> Option<&'static str> {
        self.anim.map(Anim::class)
    }
}

/// What the last render saw, so the next one can tell a change from a re-render.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Memory {
    /// The trigger, activity or visibility last seen, as a number; `None` before the first render.
    seen: Option<u32>,
    /// How many changes there have been: the restart alias.
    round: u32,
    /// Whether a `While` effect has run, so that stopping plays `hold`.
    ran: bool,
}

/// The number that stands for the effect's trigger, activity or visibility.
fn key(effect: SymbolEffect) -> u32 {
    match effect {
        SymbolEffect::Once(_, trigger) => trigger.0,
        SymbolEffect::While(_, Activity::Idle) => 0,
        SymbolEffect::While(_, Activity::Active) => 1,
        SymbolEffect::Transition(_, Shown::Hidden) => 0,
        SymbolEffect::Transition(_, Shown::Visible) => 1,
    }
}

/// The memory after rendering `effect`, and what the wrapper wears.
pub fn advance(memory: Memory, effect: SymbolEffect) -> (Memory, SymbolAttrs) {
    let now = key(effect);
    let changed = memory.seen.is_some_and(|seen| seen != now);
    let round = memory.round + u32::from(changed);
    let running = matches!(effect, SymbolEffect::While(_, Activity::Active));
    let next = Memory {
        seen: Some(now),
        round,
        ran: memory.ran || running,
    };
    let alias = Some(if round.is_multiple_of(2) { "b" } else { "a" });
    let fired = round > 0;
    let attrs = match effect {
        SymbolEffect::Once(once, _) if fired => play(once_anim(once), alias),
        SymbolEffect::Once(..) => SymbolAttrs::default(),
        SymbolEffect::While(looped, Activity::Active) => play(loop_anim(looped), Some("a")),
        SymbolEffect::While(_, Activity::Idle) if memory.ran => play(Anim::Hold, Some("a")),
        SymbolEffect::While(..) => SymbolAttrs::default(),
        SymbolEffect::Transition(TransitionEffect::Appear, Shown::Visible) if fired => {
            play(Anim::Appear, alias)
        }
        SymbolEffect::Transition(TransitionEffect::Appear, Shown::Hidden) => SymbolAttrs {
            hidden: true,
            ..SymbolAttrs::default()
        },
        SymbolEffect::Transition(TransitionEffect::Disappear, Shown::Hidden) if fired => {
            play(Anim::Disappear, alias)
        }
        SymbolEffect::Transition(TransitionEffect::Disappear, Shown::Hidden) => SymbolAttrs {
            hidden: true,
            ..SymbolAttrs::default()
        },
        SymbolEffect::Transition(TransitionEffect::Disappear, Shown::Visible) if fired => {
            play(Anim::Hold, alias)
        }
        SymbolEffect::Transition(..) => SymbolAttrs::default(),
    };
    (next, attrs)
}

fn play(anim: Anim, pulse: Option<&'static str>) -> SymbolAttrs {
    SymbolAttrs {
        anim: Some(anim),
        pulse,
        hidden: false,
    }
}

#[cfg(test)]
mod tests {
    use super::{Memory, SymbolAttrs, advance};
    use crate::anim::Anim;
    use crate::symbol::effect::{
        Activity, LoopEffect, OnceEffect, SymbolEffect, TransitionEffect, Trigger,
    };
    use ds_core::vocab::Shown;

    /// What each of a series of renders wears: its anim and alias.
    fn wear(series: &[SymbolEffect]) -> Vec<(Option<Anim>, Option<&'static str>)> {
        let mut memory = Memory::default();
        series
            .iter()
            .map(|&effect| {
                let (next, attrs) = advance(memory, effect);
                memory = next;
                (attrs.anim, attrs.pulse)
            })
            .collect()
    }

    #[test]
    fn a_once_effect_fires_on_a_change_of_trigger_and_not_on_a_render_that_changes_nothing() {
        let bounce = |trigger| SymbolEffect::Once(OnceEffect::Bounce, Trigger(trigger));
        let got = wear(&[bounce(0), bounce(0), bounce(1), bounce(1), bounce(2)]);
        let want = [
            (None, None),
            (None, None),
            (Some(Anim::Bounce), Some("a")),
            (Some(Anim::Bounce), Some("a")),
            (Some(Anim::Bounce), Some("b")),
        ];
        assert_eq!(got, want);
    }

    #[test]
    fn a_while_effect_runs_while_active_and_holds_when_it_stops() {
        let turn = |activity| SymbolEffect::While(LoopEffect::Rotate, activity);
        use Activity::{Active, Idle};
        let got = wear(&[
            turn(Idle),
            turn(Active),
            turn(Active),
            turn(Idle),
            turn(Idle),
        ]);
        let want = [
            (None, None),
            (Some(Anim::Turn), Some("a")),
            (Some(Anim::Turn), Some("a")),
            (Some(Anim::Hold), Some("a")),
            (Some(Anim::Hold), Some("a")),
        ];
        assert_eq!(got, want);
        // Active from the first render runs from the first render.
        assert_eq!(wear(&[turn(Active)]), [(Some(Anim::Turn), Some("a"))]);
    }

    #[test]
    fn every_loop_plays_its_own_animation() {
        for (effect, anim) in [
            (LoopEffect::Bounce, Anim::BounceLoop),
            (LoopEffect::Pulse, Anim::PulseLoop),
            (LoopEffect::ScaleUp, Anim::ScaleUp),
            (LoopEffect::ScaleDown, Anim::ScaleDown),
            (LoopEffect::Wiggle, Anim::WiggleLoop),
            (LoopEffect::Breathe, Anim::BreatheLoop),
            (LoopEffect::Rotate, Anim::Turn),
        ] {
            let got = wear(&[SymbolEffect::While(effect, Activity::Active)]);
            assert_eq!(got[0].0, Some(anim), "{effect:?}");
        }
    }

    #[test]
    fn a_transition_stands_at_its_first_state_and_plays_on_the_next() {
        let appear = |shown| SymbolEffect::Transition(TransitionEffect::Appear, shown);
        let disappear = |shown| SymbolEffect::Transition(TransitionEffect::Disappear, shown);
        let (visible, hidden) = (Shown::Visible, Shown::Hidden);
        let anims = |series: &[SymbolEffect]| -> Vec<Option<Anim>> {
            wear(series).into_iter().map(|(anim, _)| anim).collect()
        };
        assert_eq!(
            anims(&[
                appear(hidden),
                appear(hidden),
                appear(visible),
                appear(hidden)
            ]),
            [None, None, Some(Anim::Appear), None]
        );
        assert_eq!(
            anims(&[disappear(visible), disappear(hidden), disappear(visible)]),
            [None, Some(Anim::Disappear), Some(Anim::Hold)]
        );
    }

    #[test]
    fn an_appear_that_has_not_arrived_stands_hidden_and_a_disappear_that_never_showed_does_too() {
        let hidden = |effect| advance(Memory::default(), effect).1.hidden;
        assert!(hidden(SymbolEffect::Transition(
            TransitionEffect::Appear,
            Shown::Hidden
        )));
        assert!(hidden(SymbolEffect::Transition(
            TransitionEffect::Disappear,
            Shown::Hidden
        )));
        assert!(!hidden(SymbolEffect::Transition(
            TransitionEffect::Appear,
            Shown::Visible
        )));
        assert_eq!(
            advance(Memory::default(), SymbolEffect::NONE).1,
            SymbolAttrs::default()
        );
    }
}
