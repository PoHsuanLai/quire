//! The hook behind a pane switch: one spring carries it, `--pane-p` from 0 (the root) to 1 (the
//! detail), and the switch is moving until the spring rests on the pane asked for. Shared by the
//! preview switcher and the settings pane stack.

use crate::detail::touch::Touch;
use crate::pane_slide::{Pane, PaneRound, PaneSlide};
use crate::{
    spring::SpringPhase,
    spring_spec::SpringSpec,
    timeline::spring::{PxPerUnit, SpringFrame},
    use_spring::use_spring,
};
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// How many pixels the whole switch spans for the spring's rest: the panes move 26 px and fade,
/// so a hundredth of the switch is well under a visible step.
const SWITCH_PX: f32 = 100.0;

/// Where the spring stands for `pane`: 0 the root, 1 the detail.
fn place(pane: Pane) -> f32 {
    match pane {
        Pane::Root => 0.0,
        Pane::Detail => 1.0,
    }
}

/// The switch's state for this render and the spring's frame. The switcher is moving until the
/// spring rests on the pane asked for; a round counts each switch asked (a reversal is a new
/// one), and `on_settled` hears the pane each time the spring comes to rest on a switch.
pub fn use_pane_slide(
    shown: Pane,
    on_settled: Option<EventHandler<Pane>>,
) -> (PaneSlide, SpringFrame) {
    let spec = SpringSpec::for_touch(Touch::Remote);
    let frame = use_spring(place(shown), spec, PxPerUnit(SWITCH_PX));
    let mut asked = use_hook(|| CopyValue::new(shown));
    let mut round = use_hook(|| CopyValue::new(PaneRound::default()));
    let mut unreported = use_hook(|| CopyValue::new(None::<Pane>));
    if *asked.peek() != shown {
        asked.set(shown);
        let next = PaneRound(round.peek().0.wrapping_add(1));
        round.set(next);
        unreported.set(Some(shown));
    }
    let resting = frame.phase() == SpringPhase::Rest && frame.position() == place(shown);
    if !resting {
        let moving = PaneSlide::Moving {
            to: shown,
            round: *round.peek(),
        };
        return (moving, frame);
    }
    if *unreported.peek() == Some(shown) {
        unreported.set(None);
        if let Some(on_settled) = on_settled {
            queue_effect(move || on_settled.call(shown));
        }
    }
    (PaneSlide::rest(shown), frame)
}
