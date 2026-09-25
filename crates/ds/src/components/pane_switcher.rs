//! PaneSwitcher: a pane and its detail in one place, switched by a push (sill FINDINGS Q80;
//! design/13-BEHAVIOUR-menus-windows.md section 13.3.7: a tile's chevron opens its detail pane
//! in place, `slide-l` / `slide-r` at `--t-move --e-spring`).
//!
//! The arriving pane slides in and the outgoing one leaves the other way at the same time; the
//! leaving pane is drawn out of the flow, so the switcher's height is the arriving pane's from
//! the first frame. Both settle at one `settle()`, timed by a `MotionTimer` rather than an
//! `animationend` Blitz never sends. A switch asked for mid-slide reverses: a new round starts,
//! each pane takes the other animation from its start, and the reversed round's settle is
//! ignored (`PaneSlide`).

use crate::motion::anim::Anim;
use crate::motion::pane_slide::{Pane, PaneRole, PaneRound, PaneSlide};
use crate::motion::timer::use_motion_timer;
use crate::task::try_set;
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// Two panes, `shown` the one asked for. `on_settled` hears the pane a switch came to rest on.
#[component]
pub fn PaneSwitcher(
    shown: Pane,
    root: Element,
    detail: Element,
    #[props(default)] on_settled: Option<EventHandler<Pane>>,
) -> Element {
    let slide = use_pane_slide(shown, on_settled);
    let drawn: Vec<(Pane, PaneRole, Element)> = [(Pane::Root, root), (Pane::Detail, detail)]
        .into_iter()
        .map(|(pane, body)| (pane, slide.role(pane), body))
        .filter(|(_, role, _)| *role != PaneRole::Absent)
        .collect();
    let moving = match slide {
        PaneSlide::Moving { .. } => Some("true"),
        PaneSlide::Rest(_) => None,
    };
    rsx! {
        div { class: "ds-panes", "data-shown": slide.target().slug(), "data-moving": moving,
            for (pane , role , body) in drawn {
                div {
                    key: "{pane.slug()}",
                    class: "ds-pane",
                    "data-pane": pane.slug(),
                    "data-presence": presence(role),
                    "aria-hidden": hidden(role),
                    {body}
                }
            }
        }
    }
}

/// `data-presence` for a pane's role.
fn presence(role: PaneRole) -> &'static str {
    match role {
        PaneRole::Shown | PaneRole::Absent => "present",
        PaneRole::Arriving => "entering",
        PaneRole::Leaving => "leaving",
    }
}

/// A leaving pane is on its way out of the reading order.
fn hidden(role: PaneRole) -> Option<&'static str> {
    match role {
        PaneRole::Leaving => Some("true"),
        PaneRole::Shown | PaneRole::Arriving | PaneRole::Absent => None,
    }
}

/// The switch's state for this render: `shown` fed to the machine, and each round's settle
/// fed back from its timer. The machine lives in a plain value (it changes while rendering);
/// only the timer writes a signal, the round that settled, which this render reads.
fn use_pane_slide(shown: Pane, on_settled: Option<EventHandler<Pane>>) -> PaneSlide {
    let mut slide = use_hook(|| CopyValue::new(PaneSlide::rest(shown)));
    let mut last = use_hook(|| CopyValue::new(PaneRound::default()));
    let settled = use_signal(PaneRound::default);
    // Both panes play at `--t-move` (`PaneInR`/`PaneInL`/`PaneOutL`/`PaneOutR`, tested equal),
    // so one timer settles the pair.
    let timer = use_motion_timer(Anim::PaneInR);
    let (before, latest) = (*slide.peek(), *last.peek());
    if let Some(next) = before.show(shown, latest) {
        slide.set(next);
        if let PaneSlide::Moving { to, round } = next {
            last.set(round);
            // The handler is made here, in the component's scope; the timer starts after the
            // render, from an effect (design/05 section 7). Restarting it cancels a reversed
            // round's timer, so that settle never arrives.
            let done = EventHandler::new(move |()| {
                if try_set(settled, round).is_ok()
                    && let Some(on_settled) = on_settled
                {
                    on_settled.call(to);
                }
            });
            queue_effect(move || timer.start(done));
        }
    }
    let now = (*slide.peek()).settle(settled());
    slide.set(now);
    now
}
