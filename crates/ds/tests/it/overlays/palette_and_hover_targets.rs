//! The mail-app overlay cases: the command panel's opaque entrance, palette rows whose title
//! and detail are runs with a trailing remove, and a menu whose
//! highlight a field beside it drives; a hover target drawn as a list item or a block, and a
//! row time's tip.

use super::cases::Case;
use dioxus::prelude::*;
use ds::components::content::text_runs::RunTone;
use ds::components::menus::palette::palette_group::{PaletteGroup, PaletteRow};
use ds::components::overlays::hover_card::target::{HoverTarget, TargetElement};
use ds::host::measure::Anchor;
use ds::prelude::*;
use ds::stack::hover_hub::{HoverKey, HoverKind};
use std::time::Duration;

const NOW: Duration = Duration::ZERO;

/// A remove button that does nothing.
pub fn remove() -> RowAction {
    RowAction::new(Icon::X, "Remove from recent", EventHandler::new(|_| {}))
}

/// Recent searches: a name stronger than its path, a caller's mark, a plain title the query
/// marks, and a trailing remove on the first two.
pub fn recent_rows() -> Vec<PaletteRow<u8>> {
    vec![
        PaletteRow {
            detail: Some(TextLine::Runs(vec![
                TextRun::new("in ", RunTone::Faint),
                TextRun::new("Inbox", RunTone::Plain),
            ])),
            leading: RowLeading::Icon(Icon::Clock),
            action: Some(remove()),
            ..PaletteRow::new(
                1,
                TextLine::Runs(vec![
                    TextRun::new("from:dana ", RunTone::Strong),
                    TextRun::new("uidl", RunTone::Mark),
                ]),
            )
        },
        PaletteRow {
            leading: RowLeading::Icon(Icon::Clock),
            action: Some(remove()),
            ..PaletteRow::new(2, "invoice september")
        },
        PaletteRow::new(3, "Sync now"),
    ]
}

/// The recent rows as a menu's items.
fn recent_items() -> Vec<MenuItem<u8>> {
    recent_rows()
        .into_iter()
        .map(|row| MenuItem::new(row.value, row.title.plain_text()))
        .collect()
}

fn groups() -> Vec<PaletteGroup<u8>> {
    vec![PaletteGroup::list("Recent", recent_rows())]
}

pub const PALETTE_AND_HOVER_CASES: &[Case] = &[
    Case {
        component: "command_palette",
        state: "runs-and-trailing",
        make: || rsx! { CommandPalette { label: "Search and commands", placeholder: "Search mail, people, actions", query: "in", tokens: Vec::new(), groups: groups(), empty: "Nothing matches.", oninput: |_| {}, onpick: |_: u8| {}, onclose: |_| {} } },
        wait: NOW,
    },
    Case {
        component: "menu",
        state: "cursor-controlled",
        make: || rsx! { Menu { placement: MenuPlacement::Popup, anchor: at(), items: recent_items(), onpick: |_: u8| {}, onclose: |_| {}, active: MenuCursor::Controlled(Some(2)) } },
        wait: NOW,
    },
    Case {
        component: "menu",
        state: "cursor-none",
        make: || rsx! { Menu { placement: MenuPlacement::Popup, anchor: at(), items: recent_items(), onpick: |_: u8| {}, onclose: |_| {}, active: MenuCursor::Controlled(None) } },
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
];

/// Where the menus open.
fn at() -> Anchor {
    Anchor::Point(Point {
        x: Px(20.0),
        y: Px(20.0),
    })
}
