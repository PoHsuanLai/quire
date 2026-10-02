//! One pane of a split view: the box whose width is a spring while the pane opens or folds and
//! follows the pointer 1:1 while a divider is held. A pane whose body is an `EdgePeek`
//! (`Folded::Peeks`) stops clipping once it has folded away, so the peek can stand outside it.

use crate::components::chrome::split_view::model::Folded;
use dioxus::core::queue_effect;
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::vocab::Shown;
use ds_core::word::Word;
use ds_motion::detail::touch::Touch;
use ds_motion::spring_spec::{SpringResponse, SpringSpec};
use ds_motion::timeline::spring::PxPerUnit;
use ds_motion::use_spring::use_spring_motion;

/// Who is moving the pane's width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mover {
    /// A hand on the divider: the width follows it exactly.
    Hand,
    /// Anything else (a button, a key, a double-click): the width springs there.
    Spring,
}

/// Under this width in pixels a folding pane counts as folded away.
const FOLDED_AWAY: f32 = 0.5;

/// A pane `width` wide when open, holding `body` at that width so folding slides it away rather
/// than squeezing it.
#[component]
pub(crate) fn PaneBox(
    index: usize,
    width: Px,
    shown: Shown,
    folded: Folded,
    mover: Mover,
    body: Element,
) -> Element {
    let target = match shown {
        Shown::Visible => width.0,
        Shown::Hidden => 0.0,
    };
    let motion = use_spring_motion(target, PxPerUnit(1.0));
    let mut seen = use_hook(|| CopyValue::new(target));
    if *seen.peek() != target {
        seen.set(target);
        queue_effect(move || match mover {
            Mover::Hand => motion.track(target),
            Mover::Spring => motion.go(
                target,
                SpringSpec::for_touch(Touch::Remote).response(SpringResponse::Move),
            ),
        });
    }
    let drawn = motion.frame().position().max(0.0);
    let peeks = folded == Folded::Peeks;
    let away = peeks && shown == Shown::Hidden && drawn < FOLDED_AWAY;
    rsx! {
        div {
            class: "ds-split-pane",
            "data-index": "{index}",
            "data-shown": shown.slug(),
            "data-folded": peeks.then(|| folded.slug()),
            "data-away": if away { Some("true") } else { None },
            "aria-hidden": if shown == Shown::Hidden && !peeks { Some("true") } else { None },
            style: "--pane-w:{drawn}px;--pane-open:{width.0}px",
            div { class: "ds-split-pane-body", {body} }
        }
    }
}
