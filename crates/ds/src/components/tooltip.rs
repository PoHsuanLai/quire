//! Tooltip: text that names an action (Fly) or a value (Card) (design/04-COMPONENTS.md
//! section 18).
//!
//! Fly is pure CSS: the label sits in the target's wrapper and shows on hover after `--d-fly`,
//! at once while `.ds[data-hover=warm]` (the hover hub stamps it, O-11). Card is a small hover
//! card through the hover hub, placed below the target like a sender card.

use crate::components::hover_card::{HoverTarget, use_card};
use crate::overlay::hover_hub::{HoverKey, HoverKind, use_hover_hub};
use dioxus::prelude::*;

/// Which tooltip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TooltipKind {
    /// A small dark label above a strip button, after `--d-fly` (0 when warm).
    Fly,
    /// A small hover card for a value, through the hover hub.
    Card,
}

/// The hover key a Card tooltip files its target under: its own text, so two tips on the same
/// value share one card.
fn tip_key(text: &str) -> HoverKey {
    HoverKey(format!("tip:{text}"))
}

/// A tooltip on `children`.
#[component]
pub fn Tooltip(
    kind: TooltipKind,
    text: String,
    #[props(default)] sub: Option<String>,
    children: Element,
) -> Element {
    let hub = use_hover_hub();
    match kind {
        TooltipKind::Fly => rsx! {
            span { class: "ds-fly-target",
                {children}
                span { class: "ds-fly", role: "tooltip", "{text}" }
            }
        },
        TooltipKind::Card => {
            let key = tip_key(&text);
            let mine = hub
                .open()
                .or(hub.leaving())
                .is_some_and(|(open, _)| open == key);
            rsx! {
                HoverTarget { hover_key: key, kind: HoverKind::Sender, {children} }
                if mine {
                    TipCard { text, sub }
                }
            }
        }
    }
}

/// The Card tooltip's surface, mounted while the hub has it open.
#[component]
fn TipCard(text: String, sub: Option<String>) -> Element {
    let (float, style, presence) = use_card(HoverKind::Sender);
    let probe = float.surface();
    float.show(
        rsx! {
            div {
                class: "ds-popover ds-tip",
                "data-elevation": "pop",
                "data-layer": "card",
                "data-presence": presence,
                role: "tooltip",
                style,
                onmounted: move |event| probe.on_mounted(event),
                "{text}"
                if let Some(sub) = sub {
                    div { class: "ds-tip-sub", "{sub}" }
                }
            }
        },
        EventHandler::new(|()| {}),
    );
    rsx! {}
}
