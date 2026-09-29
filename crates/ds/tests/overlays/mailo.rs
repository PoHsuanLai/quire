//! The mail-app overlay cases: the command panel's opaque entrance, palette rows whose title
//! and detail are runs with a trailing remove, the same rows in a menu, and a menu whose
//! highlight a field beside it drives; a hover target drawn as a list item or a block, and a
//! row time's tip.

use crate::cases::Case;
use dioxus::prelude::*;
use ds::{
    Anchor, CommandPalette, HoverCard, HoverEvent, HoverKey, HoverKind, HoverTarget, Icon, Menu,
    MenuCursor, MenuEntry, MenuKind, MenuRow, MenuTile, Point, Px, RowAction, RunTone,
    TargetElement, TextLine, TextRun, use_hover_hub,
};
use std::time::Duration;

const NOW: Duration = Duration::ZERO;
/// Past the 450 ms hover intent.
const INTENT: Duration = Duration::from_millis(520);

/// A remove button that does nothing.
pub fn remove() -> RowAction {
    RowAction {
        icon: Icon::X,
        label: "Remove from recent".to_string(),
        on_press: EventHandler::new(|_| {}),
    }
}

/// Recent searches: a name stronger than its path, a caller's mark, a plain title the query
/// marks, and a trailing remove on the first two.
pub fn recent_rows() -> Vec<MenuEntry<u8>> {
    vec![
        MenuEntry::Row(MenuRow {
            detail: Some(TextLine::Runs(vec![
                TextRun::new("in ", RunTone::Faint),
                TextRun::new("Inbox", RunTone::Plain),
            ])),
            tile: Some(MenuTile::Icon(Icon::Clock)),
            trailing: Some(remove()),
            ..MenuRow::new(
                1,
                TextLine::Runs(vec![
                    TextRun::new("from:dana ", RunTone::Strong),
                    TextRun::new("uidl", RunTone::Mark),
                ]),
            )
        }),
        MenuEntry::Row(MenuRow {
            tile: Some(MenuTile::Icon(Icon::Clock)),
            trailing: Some(remove()),
            ..MenuRow::new(2, "invoice september")
        }),
        MenuRow::new(3, "Sync now").into(),
    ]
}

fn groups() -> Vec<(String, Vec<MenuEntry<u8>>)> {
    vec![("Recent".to_string(), recent_rows())]
}

pub const MAILO_CASES: &[Case] = &[
    Case {
        component: "command_palette",
        state: "runs-and-trailing",
        make: || rsx! { CommandPalette { label: "Search and commands", placeholder: "Search mail, people, actions", query: "in", tokens: Vec::new(), groups: groups(), empty: "Nothing matches.", oninput: |_| {}, onpick: |_: u8| {}, onclose: |_| {} } },
        wait: NOW,
    },
    Case {
        component: "menu",
        state: "cursor-controlled",
        make: || rsx! { Menu { kind: MenuKind::Rich, anchor: at(), entries: recent_rows(), onpick: |_: u8| {}, onclose: |_| {}, active: MenuCursor::Controlled(Some(2)) } },
        wait: NOW,
    },
    Case {
        component: "menu",
        state: "cursor-none",
        make: || rsx! { Menu { kind: MenuKind::Rich, anchor: at(), entries: recent_rows(), onpick: |_: u8| {}, onclose: |_| {}, active: MenuCursor::Controlled(None) } },
        wait: NOW,
    },
    Case {
        component: "menu",
        state: "slim-trailing",
        make: || rsx! { Menu { kind: MenuKind::Slim, anchor: at(), entries: recent_rows(), onpick: |_: u8| {}, onclose: |_| {} } },
        wait: NOW,
    },
    Case {
        component: "hover_card",
        state: "target-li",
        make: || rsx! { ul { HoverTarget { hover_key: HoverKey("side:2".to_string()), kind: HoverKind::Side, as_: TargetElement::Li, "Mei Chen" } } },
        wait: NOW,
    },
    Case {
        component: "hover_card",
        state: "target-div",
        make: || rsx! { HoverTarget { hover_key: HoverKey("thread:88".to_string()), kind: HoverKind::Thread, as_: TargetElement::Div, "Re: UIDL stability" } },
        wait: NOW,
    },
    Case {
        component: "hover_card",
        state: "tip-open",
        make: || rsx! { TimeTip {} },
        wait: INTENT,
    },
];

/// Where the menus open.
fn at() -> Anchor {
    Anchor::Point(Point {
        x: Px(20.0),
        y: Px(20.0),
    })
}

/// A row's time with its tip, open once the hub says so.
#[component]
fn TimeTip() -> Element {
    let hub = use_hover_hub();
    let key = HoverKey("time:88".to_string());
    use_hook({
        let key = key.clone();
        move || hub.feed(HoverEvent::Over((key, HoverKind::Tip)))
    });
    let open = hub.open().or(hub.leaving());
    rsx! {
        HoverTarget { hover_key: key, kind: HoverKind::Tip, "09:41" }
        if let Some((open, _)) = open {
            HoverCard { key: "{open.0}", kind: HoverKind::Tip, "Wed 23 Sep 2026, 09:41" }
        }
    }
}
