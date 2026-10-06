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
use dioxus::prelude::*;
use ds_core::word::Word;
use ds_motion::pane_slide::{Pane, PaneRole, PaneSlide};
use ds_motion::use_pane_slide::use_pane_slide;

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
