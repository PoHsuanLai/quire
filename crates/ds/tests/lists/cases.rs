//! Every list, sidebar and Space-frame component in every state, as data: the table the golden
//! test walks. Goldens live in `tests/snapshots/lists/<component>/<state>.html`.

use crate::rows::strip_actions;
use dioxus::prelude::*;
use ds::{
    Colour, CommandPill, DragGhost, DropLine, DropState, EdgePeek, Expiry, Hex, HoverStrip,
    ImageSource, MarkProvider, MarkStyle, PinFace, PinTile, Point, ProviderMark, Px, Row, RowShape,
    Selection, Shown,
};
use ds::{ControlSize, Shortcut, ShortcutKey};

/// One component in one state.
pub struct Case {
    pub component: &'static str,
    pub state: &'static str,
    pub make: fn() -> Element,
}

const VIOLET: Colour = Colour::Solid(Hex([0x5b, 0x4f, 0xc4]));

fn mark(provider: MarkProvider, size: ControlSize) -> Element {
    rsx! { ProviderMark { provider, size, style: MarkStyle::Letter } }
}

pub const CASES: &[Case] = &[
    // CommandPill.
    Case {
        component: "command_pill",
        state: "default",
        make: || rsx! { CommandPill { label: "Search or run a command", shortcut: Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char('k')]), onclick: |_| {} } },
    },
    // ProviderMark: every provider's letter across the three sizes, and a favicon.
    Case {
        component: "provider_mark",
        state: "google-tile",
        make: || mark(MarkProvider::Google, ControlSize::Regular),
    },
    Case {
        component: "provider_mark",
        state: "microsoft-row",
        make: || mark(MarkProvider::Microsoft, ControlSize::Mini),
    },
    Case {
        component: "provider_mark",
        state: "fastmail-inline",
        make: || mark(MarkProvider::Fastmail, ControlSize::Small),
    },
    Case {
        component: "provider_mark",
        state: "icloud-inline",
        make: || mark(MarkProvider::ICloud, ControlSize::Small),
    },
    Case {
        component: "provider_mark",
        state: "yahoo-row",
        make: || mark(MarkProvider::Yahoo, ControlSize::Mini),
    },
    Case {
        component: "provider_mark",
        state: "imap-tile",
        make: || mark(MarkProvider::Imap, ControlSize::Regular),
    },
    Case {
        component: "provider_mark",
        state: "image",
        make: || rsx! { ProviderMark { provider: MarkProvider::Google, size: ControlSize::Regular, style: MarkStyle::Image(ImageSource("data:image/png;base64,iVBORw0KGgo=".to_string())) } },
    },
    // PinTile: All, one selected, one not (its colour muted), nothing unread, the Add tile, a drop line.
    Case {
        component: "pin_tile",
        state: "all-pressed",
        make: || rsx! { PinTile { face: PinFace::All, selection: Selection::Selected, unread: 4, onclick: |_| {} } },
    },
    Case {
        component: "pin_tile",
        state: "one-pressed",
        make: || rsx! { PinTile { face: PinFace::Account { initial: 'P', colour: VIOLET, provider: MarkProvider::Google, address: Some("poh@acme.example".to_string()) }, selection: Selection::Selected, unread: 2, onclick: |_| {} } },
    },
    Case {
        component: "pin_tile",
        state: "one-unpressed",
        make: || rsx! { PinTile { face: PinFace::Account { initial: 'P', colour: VIOLET, provider: MarkProvider::Fastmail, address: None }, selection: Selection::Unselected, unread: 2, onclick: |_| {} } },
    },
    Case {
        component: "pin_tile",
        state: "none-unread",
        make: || rsx! { PinTile { face: PinFace::All, unread: 0, onclick: |_| {} } },
    },
    Case {
        component: "pin_tile",
        state: "drop-target",
        make: || rsx! { PinTile { face: PinFace::All, drop: DropState::Target, onclick: |_| {} } },
    },
    Case {
        component: "pin_tile",
        state: "drag-source",
        make: || rsx! { PinTile { face: PinFace::All, drop: DropState::Source, onclick: |_| {} } },
    },
    // EdgePeek: pinned in its column; hidden, with the strip at the edge waiting.
    Case {
        component: "edge_peek",
        state: "pinned",
        make: || rsx! { EdgePeek { label: "Sidebar", pinned: Shown::Visible, onpin: |_| {}, "Inbox" } },
    },
    Case {
        component: "edge_peek",
        state: "hidden",
        make: || rsx! { EdgePeek { label: "Sidebar", pinned: Shown::Hidden, onpin: |_| {}, "Inbox" } },
    },
    // A Today tab's row: what it has left, and about to expire.
    Case {
        component: "row",
        state: "today",
        make: || rsx! { Row { title: "RFC 1939", shape: RowShape::Today { left: "2 h".to_string(), expiry: Expiry::Later }, onclick: |_| {} } },
    },
    Case {
        component: "row",
        state: "today-soon",
        make: || rsx! { Row { title: "RFC 1939", shape: RowShape::Today { left: "12 min".to_string(), expiry: Expiry::Soon }, onclick: |_| {} } },
    },
    // DragGhost, DropLine.
    Case {
        component: "drag_ghost",
        state: "ghost",
        make: || rsx! { DragGhost { title: "Notes from the sync review", sub: "Sam Lindqvist", at: Point { x: Px(452.0), y: Px(206.0) } } },
    },
    Case {
        component: "drag_ghost",
        state: "drop-line",
        make: || rsx! { DropLine {} },
    },
    // HoverStrip.
    Case {
        component: "hover_strip",
        state: "default",
        make: || rsx! { HoverStrip { actions: strip_actions() } },
    },
];
