//! Overlays, mailo gaps 4: hover cards keyed on the caller's own pointer hooks, one floating
//! beside the element it measured and one drawn in place with no anchor at all; a label
//! checklist whose picks keep it open, and a menu's rows inline in a sender card.

use super::Section;
use dioxus::prelude::*;
use ds::components::vocab::Check;
use ds::{
    Button, ButtonVariant, Flow, HoverAnchor, HoverCard, HoverCardPart, HoverKey, HoverKind, Icon,
    Menu, MenuEntry, MenuKind, MenuRow, MountedRef, PickDismiss, Tile, use_hover_intent, use_rect,
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

/// The labels a thread can carry.
const LABELS: [&str; 4] = ["Invoices", "Travel", "Family", "Receipts"];

/// A label checklist: each pick toggles its check and the menu stays open.
#[component]
pub fn LabelChecklist() -> Element {
    let mut open = use_signal(|| false);
    let mut on = use_signal(|| {
        [
            Check::Checked,
            Check::Unchecked,
            Check::Unchecked,
            Check::Checked,
        ]
    });
    let trigger = use_rect();
    let rows: Vec<MenuEntry<usize>> = LABELS
        .into_iter()
        .enumerate()
        .map(|(value, name)| {
            MenuEntry::Row(MenuRow {
                check: Some(on()[value]),
                ..MenuRow::new(value, name)
            })
        })
        .collect();
    rsx! {
        Section {
            title: "A checklist that stays open",
            note: "dismiss: PickDismiss::Stay. A pick toggles its label and the menu stays, the cursor on the row; Escape or a press outside closes it.",
            div { class: "g-row",
                div { onmounted: move |event| trigger.on_mounted(event),
                    Button { variant: ButtonVariant::Secondary, label: "Labels…", onclick: move |_| open.set(true) }
                }
            }
            if let (true, Some(anchor)) = (open(), trigger.anchor()) {
                Menu::<usize> {
                    kind: MenuKind::Dropdown,
                    anchor,
                    entries: rows,
                    onpick: move |value: usize| {
                        on.with_mut(|on| {
                            on[value] = match on[value] {
                                Check::Checked => Check::Unchecked,
                                Check::Unchecked => Check::Checked,
                            }
                        })
                    },
                    onclose: move |()| open.set(false),
                    dismiss: PickDismiss::Stay,
                }
            }
        }
    }
}

/// A sender card's actions, drawn as a menu's rows inside the card.
#[component]
pub fn InlineActions() -> Element {
    let mut said = use_signal(|| "nothing yet".to_string());
    let actions: Vec<MenuEntry<&'static str>> = [
        (Icon::Pin, "Pin Dana"),
        (Icon::Search, "Every thread from Dana"),
        (Icon::Copy, "Copy address"),
    ]
    .into_iter()
    .map(|(icon, title)| {
        MenuEntry::Row(MenuRow {
            tile: Some(Tile::Icon(icon)),
            ..MenuRow::new(title, title)
        })
    })
    .collect();
    rsx! {
        Section {
            title: "Menu rows inline in a card",
            note: "flow: Flow::Inline draws the same rows in the caller's card: no overlay, no scrim, no layer on the stack, no focus taken.",
            HoverCard {
                kind: HoverKind::Sender,
                flow: Flow::Inline,
                parts: vec![
                    HoverCardPart::Title("Dana Okafor".to_string()),
                    HoverCardPart::Sub("dana@example.org".to_string()),
                ],
                Menu::<&'static str> {
                    kind: MenuKind::Rich,
                    anchor: ds::Anchor::Point(ds::Point::default()),
                    entries: actions,
                    onpick: move |title: &'static str| said.set(title.to_string()),
                    onclose: |()| {},
                    flow: Flow::Inline,
                }
            }
            p { class: "g-note", "Picked: {said}" }
        }
    }
}
