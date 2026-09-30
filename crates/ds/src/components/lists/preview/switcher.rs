//! PaneSwitcher: a pane and its detail in one place, switched by a push
//! (design/13-BEHAVIOUR-menus-windows.md section 13.3.7: a tile's chevron opens its detail pane
//! in place).
//!
//! The arriving pane slides in and the outgoing one leaves the other way at the same time; the
//! leaving pane is drawn out of the flow, so the switcher's height is the arriving pane's from
//! the first frame. Driven motion (design/05 section 14): one spring in Rust carries
//! the switch, `--pane-p` from 0 (the root) to 1 (the detail), and both panes are drawn from it.
//! A switch asked for mid-slide redirects that spring from where it is at the speed it has, so
//! the panes turn back without a jump, and the switcher rests (and `on_settled` hears the pane)
//! when the spring does. Under Reduced the spring is critically damped and the panes only
//! cross-fade.
use crate::motion::detail::touch::Touch;
use crate::motion::pane_slide::{Pane, PaneRole, PaneRound, PaneSlide};
use crate::motion::{
    spring::SpringPhase,
    spring_spec::SpringSpec,
    timeline::spring::{PxPerUnit, SpringFrame},
    use_spring::use_spring,
};
use dioxus::core::queue_effect;
use dioxus::prelude::*;
use ds_core::word::Word;

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

/// Two panes, `shown` the one asked for. `on_settled` hears the pane a switch came to rest on.
#[component]
pub fn PaneSwitcher(
    shown: Pane,
    root: Element,
    detail: Element,
    #[props(default)] on_settled: Option<EventHandler<Pane>>,
) -> Element {
    let (slide, frame) = use_pane_slide(shown, on_settled);
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
        div {
            class: "ds-panes",
            "data-shown": slide.target().slug(),
            "data-moving": moving,
            style: "--pane-p:{frame.css()}",
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

/// The switch's state for this render and the spring's frame. The switcher is moving until the
/// spring rests on the pane asked for; a round counts each switch asked (a reversal is a new
/// one), and `on_settled` hears the pane each time the spring comes to rest on a switch.
fn use_pane_slide(shown: Pane, on_settled: Option<EventHandler<Pane>>) -> (PaneSlide, SpringFrame) {
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
