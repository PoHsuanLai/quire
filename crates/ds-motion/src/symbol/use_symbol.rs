//! The hooks under `Symbol`: the wrapper's attributes, and the part driver's progress.

use super::attrs::{Memory, SymbolAttrs, advance};
use super::effect::{Activity, SymbolEffect};
use super::route::{PartRun, Route};
use super::timing::cycle;
use crate::detail::level::use_level;
use crate::timeline::cycle::Cycle;
use crate::timeline::playback::use_playback;
use dioxus::core::queue_effect;
use dioxus::prelude::*;
use ds_style::appearance::motion::MotionLevel;
use ds_style::icon::posed::Thousandths;

/// What the wrapper wears for `effect`'s stylesheet share: an `a-<anim>` class and the restart
/// alias. It changes on a change of the effect's trigger, activity or visibility, never on a
/// render that changes nothing. A button whose icon turns while it is busy wears exactly this.
pub fn use_symbol_attrs(effect: SymbolEffect) -> SymbolAttrs {
    let mut memory = use_hook(|| CopyValue::new(Memory::default()));
    let (next, attrs) = advance(*memory.peek(), effect);
    if *memory.peek() != next {
        memory.set(next);
    }
    attrs
}

/// What the part driver does when `run` follows `was`, at `level`: the cycle to play, or `None`
/// to leave things as they are.
fn plan(was: Option<PartRun>, run: PartRun, gesture: PlanFor, level: MotionLevel) -> Option<Cycle> {
    let rest = was.map(|_| Cycle::REST);
    if level == MotionLevel::Reduced {
        return rest;
    }
    match (was, run) {
        (None, PartRun::Once(_)) => None,
        (Some(PartRun::Once(before)), PartRun::Once(now)) if before == now => None,
        (_, PartRun::Once(_)) => Some(cycle(gesture.0, run, level)),
        (_, PartRun::While(Activity::Active)) => Some(cycle(gesture.0, run, level)),
        (_, PartRun::While(Activity::Idle)) => rest,
    }
}

/// The gesture a plan is for, as a type of its own so `plan`'s arguments read apart.
#[derive(Debug, Clone, Copy)]
struct PlanFor(ds_style::icon::parts::PartGesture);

/// How far through its cycle the part driver is, in thousandths: 1000 at rest. A `Once` run plays
/// when its trigger changes (never on the first render), a `While` run plays from the moment it
/// is active until it is not, and under Reduced nothing plays (R7).
pub fn use_part_progress(route: Route) -> Thousandths {
    let env = use_level();
    let play = use_playback(Cycle::REST);
    let mut seen = use_hook(|| CopyValue::new(None::<PartRun>));
    if let Route::Parts(gesture, run) = route {
        let was = *seen.peek();
        if was != Some(run) {
            seen.set(Some(run));
            if let Some(cycle) = plan(was, run, PlanFor(gesture), env.now()) {
                queue_effect(move || play.play(cycle));
            }
        }
    }
    Thousandths(i32::try_from(play.frame()).unwrap_or(1000))
}
