//! Overlays, mailo gaps 4: hover cards keyed on the caller's own pointer hooks, one floating
//! beside the element it measured and one drawn in place with no anchor at all.

use super::Section;
use dioxus::prelude::*;
use ds::{
    Button, ButtonVariant, Flow, HoverAnchor, HoverCard, HoverCardPart, HoverKey, HoverKind,
    MountedRef, use_hover_intent,
};

/// The prefix of this section's hover keys: the page's other card section skips them.
pub const HOOK_KEYED: &str = "hook:";

/// How a hook-keyed demo target places its card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Placing {
    /// Against its own element, measured.
    Element,
    /// Unplaced, drawn in the slot below.
    Unplaced,
}

/// A target that feeds the hub from its own hooks.
#[component]
fn Keyed(name: &'static str, placing: Placing) -> Element {
    let driver = use_hover_intent();
    let mut element = use_signal(|| None::<MountedRef>);
    let key = HoverKey(format!("{HOOK_KEYED}{name}"));
    rsx! {
        span {
            onmounted: move |event| element.set(Some(MountedRef(event.data()))),
            onpointerenter: move |_| {
                let anchor = match (placing, element.peek().clone()) {
                    (Placing::Element, Some(mounted)) => HoverAnchor::Element(mounted),
                    (Placing::Element, None) | (Placing::Unplaced, _) => HoverAnchor::Unplaced,
                };
                driver.over(key.clone(), HoverKind::Sender, anchor);
            },
            onpointerleave: move |_| driver.out(),
            Button { variant: ButtonVariant::Quiet, label: name, onclick: |_| {} }
        }
    }
}

/// The card for an open hook-keyed key.
fn card(key: &HoverKey, flow: Flow) -> Element {
    let name = key.0.trim_start_matches(HOOK_KEYED).to_string();
    rsx! {
        HoverCard {
            key: "{key.0}",
            kind: HoverKind::Sender,
            flow,
            parts: vec![
                HoverCardPart::Title(name),
                HoverCardPart::Sub("keyed on the caller's pointer hooks".to_string()),
            ],
        }
    }
}

/// Two hook-keyed targets: one card floats beside its target, the other stands in the slot.
#[component]
pub fn HookKeyedCards() -> Element {
    let hub = use_hover_intent().hub();
    let open = hub
        .open()
        .or(hub.leaving())
        .filter(|(key, _)| key.0.starts_with(HOOK_KEYED));
    let floating = open
        .as_ref()
        .filter(|(key, _)| key.0.ends_with("(element)"))
        .map(|(key, _)| card(key, Flow::Floating));
    let inline = open
        .as_ref()
        .filter(|(key, _)| key.0.ends_with("(unplaced)"))
        .map(|(key, _)| card(key, Flow::Inline));
    rsx! {
        Section {
            title: "Cards keyed on the caller's hooks",
            note: "use_hover_intent drives the same 450/150/400 ms machine from a caller's own pointer events. The first card floats beside the element it measured; the second has no anchor and is drawn in place (flow: Flow::Inline), as a test with no layout sees it.",
            div { class: "g-row",
                Keyed { name: "Dana Okafor (element)", placing: Placing::Element }
                Keyed { name: "Sam Lindqvist (unplaced)", placing: Placing::Unplaced }
            }
            div { class: "g-row g-stage-pad", {inline} }
            {floating}
        }
    }
}
