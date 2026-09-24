//! The mailo gaps 4 overlay cases: hover cards keyed on the caller's own hooks through
//! `use_hover_intent`, drawn in place with no anchor or floating against a rect the caller
//! already had; a label checklist whose picks keep it open, and a menu's rows drawn inline in
//! a sender card.

use crate::cases::Case;
use dioxus::prelude::*;
use ds::components::vocab::Check;
use ds::{
    Anchor, Flow, HoverAnchor, HoverCard, HoverCardPart, HoverKey, HoverKind, Icon, Menu,
    MenuEntry, MenuKind, MenuRow, PickDismiss, Point, Px, Rect, Size, Tile, use_hover_intent,
};
use std::time::Duration;

/// Past the 450 ms hover intent.
const INTENT: Duration = Duration::from_millis(520);

const NOW: Duration = Duration::ZERO;

pub const MAILO4_CASES: &[Case] = &[
    Case {
        component: "menu",
        state: "stay-checklist",
        make: || rsx! { Menu { kind: MenuKind::Dropdown, anchor: at(), entries: labels(), onpick: |_: u8| {}, onclose: |_| {}, dismiss: PickDismiss::Stay } },
        wait: NOW,
    },
    Case {
        component: "menu",
        state: "inline-rich",
        make: || rsx! { div { Menu { kind: MenuKind::Rich, anchor: at(), entries: sender_actions(), onpick: |_: u8| {}, onclose: |_| {}, flow: Flow::Inline } } },
        wait: NOW,
    },
    Case {
        component: "menu",
        state: "inline-checklist",
        make: || rsx! { div { Menu { kind: MenuKind::Dropdown, anchor: at(), entries: labels(), onpick: |_: u8| {}, onclose: |_| {}, flow: Flow::Inline, dismiss: PickDismiss::Stay } } },
        wait: NOW,
    },
    Case {
        component: "hover_card",
        state: "hook-keyed-inline",
        make: || rsx! { HookKeyed { anchor: HoverAnchor::Unplaced, flow: Flow::Inline } },
        wait: INTENT,
    },
    Case {
        component: "hover_card",
        state: "hook-keyed-unplaced",
        make: || rsx! { HookKeyed { anchor: HoverAnchor::Unplaced, flow: Flow::Floating } },
        wait: INTENT,
    },
    Case {
        component: "hover_card",
        state: "hook-keyed-rect",
        make: || rsx! { HookKeyed { anchor: HoverAnchor::Rect(name_rect()), flow: Flow::Floating } },
        wait: INTENT,
    },
];

/// Where the floating menus open.
fn at() -> Anchor {
    Anchor::Point(Point {
        x: Px(20.0),
        y: Px(20.0),
    })
}

/// A label checklist: two labels on, one off.
fn labels() -> Vec<MenuEntry<u8>> {
    [
        ("Invoices", Check::Checked),
        ("Travel", Check::Unchecked),
        ("Family", Check::Checked),
    ]
    .into_iter()
    .zip(0u8..)
    .map(|((name, check), value)| {
        MenuEntry::Row(MenuRow {
            check: Some(check),
            ..MenuRow::new(value, name)
        })
    })
    .collect()
}

/// A sender card's actions.
fn sender_actions() -> Vec<MenuEntry<u8>> {
    vec![
        MenuEntry::Row(MenuRow {
            tile: Some(Tile::Icon(Icon::Pin)),
            ..MenuRow::new(0, "Pin Dana")
        }),
        MenuEntry::Row(MenuRow {
            tile: Some(Tile::Icon(Icon::Search)),
            ..MenuRow::new(1, "Every thread from Dana")
        }),
    ]
}

/// Where a row's sender name was when the pointer came over it.
fn name_rect() -> Rect {
    Rect {
        origin: Point {
            x: Px(120.0),
            y: Px(40.0),
        },
        size: Size {
            width: Px(90.0),
            height: Px(18.0),
        },
    }
}

/// A sender card opened from the caller's own hook (here, as the page mounts), with no
/// target element of quire's anywhere.
#[component]
fn HookKeyed(anchor: HoverAnchor, flow: Flow) -> Element {
    let driver = use_hover_intent();
    use_hook(move || driver.over(HoverKey("sender:3".to_string()), HoverKind::Sender, anchor));
    let hub = driver.hub();
    let open = hub.open().or(hub.leaving());
    rsx! {
        div {
            if let Some((key, kind)) = open {
                HoverCard {
                    key: "{key.0}",
                    kind,
                    flow,
                    parts: vec![
                        HoverCardPart::Title("Dana Okafor".to_string()),
                        HoverCardPart::Sub("dana@example.org".to_string()),
                    ],
                }
            }
        }
    }
}
