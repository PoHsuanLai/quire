//! Reveal: a list's children rise in turn on first show, and never again (design/26-DETAILS.md
//! R1, R13).

use super::first_show::FirstShow;
use crate::components::vocab::StaggerIndex;
use crate::motion::{Anim, TimerPhase, use_motion_timer};
use dioxus::prelude::*;

/// Its direct children play `rise` at `--t-move --e-out`, each `--stagger` after the one before,
/// the index capped at 12 (R13), only when `first` is `Animate` and only in the first settle
/// after mounting; a re-render never replays it (R1). Under Reduced the stagger is 0 and the rise
/// 60 ms, as the level's tokens say. The wrapper is a plain block (`div.ds-reveal`): lay the
/// children out inside it.
#[component]
pub fn Reveal(first: FirstShow, children: Element) -> Element {
    let timer = use_motion_timer(Anim::Rise);
    use_hook(|| {
        if first == FirstShow::Animate {
            timer.start_staggered(StaggerIndex::new(usize::from(StaggerIndex::CAP)));
        }
    });
    let playing = match timer.phase() {
        TimerPhase::Running => "play",
        TimerPhase::Idle | TimerPhase::Settled => "still",
    };
    rsx! {
        div { class: "ds-reveal", "data-reveal": playing, {children} }
    }
}
