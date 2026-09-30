//! The App features page (design/30 section 2.11): the pinned tiles with their drop line, the
//! Today tabs that expire, the edge-peek sidebar, the link pill, the launcher's grouped commands
//! in a Space's order, and Space switching by Control and a digit.

use super::Section;
use dioxus::prelude::*;
use ds::time::now;
use ds::{
    Accessory, Button, Colour, CommandPalette, CommandPaletteHost, Corner, DropState, EdgePeek,
    GroupOrder, Hex, Icon, LinkPill, LinkTarget, MarkProvider, Material, PaletteGroup, PaletteRow,
    PinAdd, PinFace, PinItem, PinTile, PinTiles, Radius, RowLeading, Selection, Shown, Surface,
    TodayTab, TodayTabs, space_pressed, space_shortcut,
};
use ds::{ControlSize, KeyEquivalent, KeyStyle};
use std::time::Duration;

/// The App features page.
#[component]
pub fn AppFeaturesPage() -> Element {
    rsx! {
        PinnedTiles {}
        Today {}
        Edge {}
        Link {}
        Grouped {}
        Spaces {}
    }
}

fn account(initial: char, provider: MarkProvider) -> PinFace {
    PinFace::Account {
        initial,
        colour: Colour::Solid(Hex([0x1a, 0x73, 0xe8])),
        provider,
        address: Some(format!("{initial}@example.org").to_lowercase()),
    }
}

/// Every face and state of a tile, and the live grid: press a tile to filter, drag one onto
/// another's place to reorder (the accent line shows where it lands).
#[component]
fn PinnedTiles() -> Element {
    let mut order = use_signal(|| vec!['P', 'W', 'G', 'L']);
    let mut selected = use_signal(|| Some('P'));
    let items: Vec<PinItem<char>> = order()
        .into_iter()
        .map(|key| PinItem {
            key,
            face: account(
                key,
                match key {
                    'P' => MarkProvider::Google,
                    'W' => MarkProvider::Microsoft,
                    'G' => MarkProvider::Fastmail,
                    _ => MarkProvider::Local,
                },
            ),
            unread: match key {
                'P' => 3,
                'W' => 120,
                _ => 0,
            },
        })
        .collect();
    rsx! {
        Section { title: "Pinned tiles", note: "PinTile: All, an account selected and not (its colour muted), a count, the drop line before a tile a drag would take the place of, the dragged tile dimmed, and the Add tile. Below, the live PinTiles: press to select, drag a tile onto another to reorder.",
            div { class: "g-row",
                PinTile { face: PinFace::All, selection: Selection::Selected, unread: 7, onclick: |_| {} }
                PinTile { face: account('P', MarkProvider::Google), selection: Selection::Selected, unread: 3, onclick: |_| {} }
                PinTile { face: account('W', MarkProvider::Microsoft), unread: 0, onclick: |_| {} }
                PinTile { face: account('G', MarkProvider::Fastmail), drop: DropState::Target, unread: 1, onclick: |_| {} }
                PinTile { face: account('L', MarkProvider::Local), drop: DropState::Source, onclick: |_| {} }
                PinTile { face: PinFace::Add { label: "Add account".to_string(), hint: Some("Add account…".to_string()) }, onclick: |_| {} }
            }
            div { style: "width:260px",
                PinTiles {
                    label: "Accounts",
                    items,
                    selected: selected(),
                    onpick: move |key| selected.set(Some(key)),
                    onreorder: move |next| order.set(next),
                    add: PinAdd { label: "Add account".to_string(), hint: None, onadd: EventHandler::new(|()| {}) },
                }
            }
            p { class: "g-note", "Order: {order().iter().collect::<String>()}" }
        }
    }
}

/// Tabs with 3 h, 20 min and 90 s left. The one about to go is drawn quieter, and each leaves by
/// the roster when its close is pressed or its time is up.
#[component]
fn Today() -> Element {
    let mut selected = use_signal(|| Some(1u8));
    let mut tabs = use_signal(|| {
        let start = now();
        vec![
            (1u8, "RFC 1939: POP3", 3 * 3600),
            (2, "UIDL stability", 20 * 60),
            (3, "Sync review notes", 90),
        ]
        .into_iter()
        .map(|(key, title, secs)| TodayTab {
            key,
            title: title.to_string(),
            leading: RowLeading::Icon(Icon::Globe),
            expires: start + Duration::from_secs(secs),
        })
        .collect::<Vec<_>>()
    });
    rsx! {
        Section { title: "Today tabs", note: "Row shaped Today in a SourceList: what each has left is trailing data, one under half an hour is quieter, the close button removes a tab, and a tab whose time runs out leaves by itself (watch the last one).",
            div { style: "width:240px",
                TodayTabs {
                    label: "Today",
                    tabs: tabs(),
                    selected: selected(),
                    onpick: move |key| selected.set(Some(key)),
                    onclose: move |key| tabs.with_mut(|tabs| tabs.retain(|tab| tab.key != key)),
                    onexpire: move |key| tabs.with_mut(|tabs| tabs.retain(|tab| tab.key != key)),
                }
            }
        }
    }
}

/// A stage with a pinned sidebar or an edge that peeks it: rest the pointer on the left edge.
#[component]
fn Edge() -> Element {
    let mut pinned = use_signal(|| Shown::Hidden);
    let columns = match pinned() {
        Shown::Visible => "226px 1fr",
        Shown::Hidden => "0px 1fr",
    };
    rsx! {
        Section { title: "Edge-peek sidebar", note: "Hidden, the 10 px strip at the left edge waits; after a short rest the sidebar slides in from the left by a spring, floating over the content, and slides out when the pointer leaves it. A click on the strip pins it into its column.",
            div { class: "g-row",
                Button {
                    size: ControlSize::Mini,
                    label: match pinned() { Shown::Visible => "Hide the sidebar", Shown::Hidden => "Pin the sidebar" },
                    onclick: move |_| pinned.set(pinned().flipped()),
                }
            }
            div { class: "g-stage", "data-wide": "true",
                div { class: "g-side-host", style: "grid-template-columns:{columns}",
                    EdgePeek {
                        label: "Sidebar",
                        pinned: pinned(),
                        onpin: move |()| pinned.set(Shown::Visible),
                        div { class: "g-side-body",
                            p { class: "g-name", "Inbox" }
                            p { class: "g-name", "Starred" }
                            p { class: "g-name", "Sent" }
                        }
                    }
                    div { class: "g-side-content", "Content" }
                }
            }
        }
    }
}

/// The pill honest and lying: rest the pointer on one to see the whole address; press to copy.
#[component]
fn Link() -> Element {
    let mut copied = use_signal(String::new);
    rsx! {
        Section { title: "Link pill", note: "Collapsed it names the registered domain; resting the pointer on it expands to the whole address (a lying link says where it goes and what the text claimed); a press hands the address to the caller and the pill says Copied until the pointer leaves.",
            div { class: "g-stage-row",
                div { class: "g-stage",
                    LinkPill {
                        href: "https://docs.example.org/guides/imap/uidplus",
                        oncopy: move |href: String| copied.set(href),
                        target: LinkTarget::Honest {
                            scheme_sub: "https://docs.".to_string(),
                            registered: "example.org".to_string(),
                            path: "/guides/imap/uidplus".to_string(),
                        },
                    }
                }
                div { class: "g-stage",
                    LinkPill {
                        href: "https://examp1e-login.net/",
                        oncopy: |_| {},
                        target: LinkTarget::Lying { registered: "examp1e-login.net".to_string(), shown: "example.org".to_string() },
                    }
                }
            }
            p { class: "g-note", "Last copied: {copied()}" }
        }
    }
}

/// The kinds of result the launcher groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Apps,
    Files,
    Commands,
}

fn group(kind: Kind) -> (Kind, PaletteGroup<u8>) {
    let (title, rows): (&str, Vec<(u8, &str, Icon)>) = match kind {
        Kind::Apps => (
            "Apps",
            vec![(1, "Mail", Icon::Mail), (2, "Calendar", Icon::Clock)],
        ),
        Kind::Files => ("Files", vec![(3, "Quarterly report.pdf", Icon::Folder)]),
        Kind::Commands => (
            "Commands",
            vec![(4, "Lock", Icon::Lock), (5, "Log Out", Icon::Power)],
        ),
    };
    let rows = rows
        .into_iter()
        .map(|(value, title, icon)| PaletteRow {
            leading: RowLeading::Icon(icon),
            accessory: Accessory::None,
            ..PaletteRow::new(value, title)
        })
        .collect();
    (kind, PaletteGroup::list(title, rows))
}

/// Two Spaces, two orders: the same results, grouped by kind under their section headers.
#[component]
fn Grouped() -> Element {
    let panel = |label: &'static str, order: Vec<Kind>| {
        let groups = GroupOrder(order).arrange(vec![
            group(Kind::Apps),
            group(Kind::Files),
            group(Kind::Commands),
        ]);
        rsx! {
            div { class: "g-launcher",
                Surface { material: Material::Sheet, radius: Some(Corner::Token(Radius::Panel)),
                    CommandPalette::<u8> {
                        label,
                        placeholder: label,
                        query: String::new(),
                        tokens: Vec::new(),
                        groups,
                        empty: "Nothing matches.",
                        oninput: |_| {},
                        onpick: |_| {},
                        onclose: |_| {},
                        host: CommandPaletteHost::Surface,
                    }
                }
            }
        }
    };
    rsx! {
        Section { title: "Grouped launcher commands", note: "CommandPalette groups its results by kind, each under a SectionHeader; GroupOrder puts a Space's kinds first. Left: the Work Space (apps, files, commands). Right: the Home Space (commands first).",
            div { class: "g-row g-row-top",
                {panel("Work", vec![Kind::Apps, Kind::Files, Kind::Commands])}
                {panel("Home", vec![Kind::Commands, Kind::Apps])}
            }
        }
    }
}

/// Focus the field and press Control and a digit: the swatch takes that Space's colour and
/// cross-fades over `--t-big`.
#[component]
fn Spaces() -> Element {
    let mut space = use_signal(|| 1u8);
    rsx! {
        Section { title: "Space switching", note: "Control and 1 to 9 switch to that Space (Command and a digit are left to the apps). The frame's colour cross-fades over --t-big.",
            div { class: "g-row",
                div {
                    class: "g-space-key",
                    tabindex: "0",
                    "data-space": "{space}",
                    onkeydown: move |event| {
                        if let Some(next) = space_pressed(&event.key(), event.modifiers()) {
                            event.prevent_default();
                            space.set(next.get());
                        }
                    },
                    "Focus here, then press ⌃1 to ⌃9: Space {space}"
                }
                for n in 1u8..=3 {
                    if let Some(number) = ds::SpaceNumber::new(n) {
                        KeyEquivalent { shortcut: space_shortcut(number), style: KeyStyle::Cap, size: ControlSize::Small }
                    }
                }
            }
        }
    }
}
