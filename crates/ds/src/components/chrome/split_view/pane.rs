//! One pane of a split view: the box whose width is a spring while the pane opens or folds and
//! follows the pointer 1:1 while a divider is held. A pane whose body is an `EdgePeek`
//! (`Folded::Peeks`) stops clipping once it has folded away, so the peek can stand outside it.

use crate::components::chrome::split_view::model::{Folded, PaneFocus};
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
/// than squeezing it. When the view is too narrow it gives up width down to `least`, and its
/// body follows it there.
#[component]
pub(crate) fn PaneBox(
    index: usize,
    width: Px,
    least: Px,
    shown: Shown,
    folded: Folded,
    mover: Mover,
    focus: PaneFocus,
    on_hold: EventHandler<()>,
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
    // Never held wider than it is drawn, so a folding pane's spring is not stopped at its least.
    let held = least.0.min(drawn);
    rsx! {
        div {
            class: "ds-split-pane",
            "data-index": "{index}",
            "data-shown": shown.slug(),
            "data-focus": focus.attribute(),
            onpointerdown: move |_| on_hold.call(()),
            onkeydown: move |_| on_hold.call(()),
            "data-folded": peeks.then(|| folded.slug()),
            "data-away": if away { Some("true") } else { None },
            "aria-hidden": if shown == Shown::Hidden && !peeks { Some("true") } else { None },
            style: "--pane-w:{drawn}px;--pane-open:{width.0}px;--pane-least:{held}px",
            div { class: "ds-split-pane-body", {body} }
        }
    }
}
