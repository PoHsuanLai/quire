//! Tooltip: text that names an action (Fly) or a value (Card) (design/04-COMPONENTS.md
//! section 18).
//!
//! Fly is pure CSS: the label sits in the target's wrapper and shows on hover after `--d-fly`,
//! at once while `.ds[data-hover=warm]` (the hover hub stamps it, O-11). Card is a small hover
//! card through the hover hub, placed below the target like a sender card.
//!
//! A caller that runs its own machine (the dock's label rules: hide on press, while a menu is
//! open, while dragging; sill FINDINGS Q17) passes `shown`, and the tooltip shows or hides on
//! its say alone, at once, with no hover and no delay of its own.

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

/// Whether a caller-driven tooltip is up: `data-shown` on the Fly's target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Shown {
    /// Up, at once.
    Visible,
    /// Down, even under the pointer.
    Hidden,
}

impl Shown {
    /// The `data-shown` word.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            Shown::Visible => "visible",
            Shown::Hidden => "hidden",
        }
    }
}

/// The hover key a Card tooltip files its target under: its own text, so two tips on the same
/// value share one card.
fn tip_key(text: &str) -> HoverKey {
    HoverKey(format!("tip:{text}"))
}

/// A tooltip on `children`. `shown` hands it to the caller: `None` follows the pointer (the
/// Fly's `:hover` and `--d-fly`, the Card's hover hub), `Some` shows or hides it at once.
#[component]
pub fn Tooltip(
    kind: TooltipKind,
    text: String,
    #[props(default)] sub: Option<String>,
    #[props(default)] shown: Option<Shown>,
    children: Element,
) -> Element {
    let hub = use_hover_hub();
    match kind {
        TooltipKind::Fly => rsx! {
            span { class: "ds-fly-target", "data-shown": shown.map(Shown::slug),
                {children}
                span { class: "ds-fly", role: "tooltip", "{text}" }
            }
        },
        TooltipKind::Card => {
            let key = tip_key(&text);
            let hovered = || {
                hub.open()
                    .or(hub.leaving())
                    .is_some_and(|(open, _)| open == key)
            };
            let mine = match shown {
                Some(Shown::Visible) => true,
                Some(Shown::Hidden) => false,
                None => hovered(),
            };
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
