//! Overlays, for mail: hover cards keyed on the caller's own pointer hooks, one floating
//! beside the element it measured and one drawn in place with no anchor at all, and a menu's
//! items inline in a sender card.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::Bezel;
use ds::{
    Button, Flow, HoverAnchor, HoverCard, HoverCardPart, HoverKey, HoverKind, HoverProfile, Icon,
    Menu, MenuImage, MenuItem, MenuPlacement, MountedRef, use_hover_intent,
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
                driver.over(key.clone(), HoverProfile::Card, anchor);
            },
            onpointerleave: move |_| driver.out(),
            Button { bezel: Bezel::Inline, label: name, onclick: |_| {} }
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

/// A sender card's actions, drawn as a menu's rows inside the card.
#[component]
pub fn InlineActions() -> Element {
    let mut said = use_signal(|| "nothing yet".to_string());
    let actions: Vec<MenuItem<&'static str>> = [
        (Icon::Pin, "Pin Dana"),
        (Icon::Search, "Every thread from Dana"),
        (Icon::Copy, "Copy address"),
    ]
    .into_iter()
    .map(|(icon, title)| MenuItem::new(title, title).with_image(MenuImage::Icon(icon)))
    .collect();
    rsx! {
        Section {
            title: "Menu items inline in a card",
            note: "flow: Flow::Inline draws the same items in the caller's card: no overlay, no scrim, no layer on the stack, no focus taken.",
            HoverCard {
                kind: HoverKind::Sender,
                flow: Flow::Inline,
                parts: vec![
                    HoverCardPart::Title("Dana Okafor".to_string()),
                    HoverCardPart::Sub("dana@example.org".to_string()),
                ],
                Menu::<&'static str> {
                    placement: MenuPlacement::Popup,
                    anchor: ds::Anchor::Point(ds::Point::default()),
                    items: actions,
                    onpick: move |title: &'static str| said.set(title.to_string()),
                    onclose: |()| {},
                    flow: Flow::Inline,
                }
            }
            p { class: "g-note", "Picked: {said}" }
        }
    }
}
