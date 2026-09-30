//! Every list, sidebar and Space-frame component in every state, as data: the table the golden
//! test walks. Goldens live in `tests/snapshots/lists/<component>/<state>.html`.

use crate::rows::strip_actions;
use dioxus::prelude::*;
use ds::{Accent, Appearance, AppearancePicker, Motion, ReducedMotion, Scheme, SystemPrefs, Theme};
use ds::{
    AccountFace, AccountTile, Colour, CommandPill, DragGhost, DropLine, EdgeStrip, Grip, Hex,
    HoverStrip, ImageSource, MarkProvider, MarkSize, MarkStyle, Point, ProviderMark, Px,
};
use ds::{Check, Shortcut, ShortcutKey};

/// One component in one state.
pub struct Case {
    pub component: &'static str,
    pub state: &'static str,
    pub make: fn() -> Element,
}

const VIOLET: Colour = Colour::Solid(Hex([0x5b, 0x4f, 0xc4]));

fn mark(provider: MarkProvider, size: MarkSize) -> Element {
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
        make: || mark(MarkProvider::Google, MarkSize::Tile),
    },
    Case {
        component: "provider_mark",
        state: "microsoft-row",
        make: || mark(MarkProvider::Microsoft, MarkSize::Row),
    },
    Case {
        component: "provider_mark",
        state: "fastmail-inline",
        make: || mark(MarkProvider::Fastmail, MarkSize::Inline),
    },
    Case {
        component: "provider_mark",
        state: "icloud-inline",
        make: || mark(MarkProvider::ICloud, MarkSize::Inline),
    },
    Case {
        component: "provider_mark",
        state: "yahoo-row",
        make: || mark(MarkProvider::Yahoo, MarkSize::Row),
    },
    Case {
        component: "provider_mark",
        state: "imap-tile",
        make: || mark(MarkProvider::Imap, MarkSize::Tile),
    },
    Case {
        component: "provider_mark",
        state: "image",
        make: || rsx! { ProviderMark { provider: MarkProvider::Google, size: MarkSize::Tile, style: MarkStyle::Image(ImageSource("data:image/png;base64,iVBORw0KGgo=".to_string())) } },
    },
    // AccountTile: All, one pressed, one unpressed (desaturated), nothing unread.
    Case {
        component: "account_tile",
        state: "all-pressed",
        make: || rsx! { AccountTile { account: AccountFace::All, pressed: Check::On, unread: 4, onclick: |_| {} } },
    },
    Case {
        component: "account_tile",
        state: "one-pressed",
        make: || rsx! { AccountTile { account: AccountFace::One { initial: 'P', colour: VIOLET, provider: MarkProvider::Google, address: Some("poh@acme.example".to_string()) }, pressed: Check::On, unread: 2, onclick: |_| {} } },
    },
    Case {
        component: "account_tile",
        state: "one-unpressed",
        make: || rsx! { AccountTile { account: AccountFace::One { initial: 'P', colour: VIOLET, provider: MarkProvider::Fastmail, address: None }, pressed: Check::Off, unread: 2, onclick: |_| {} } },
    },
    Case {
        component: "account_tile",
        state: "none-unread",
        make: || rsx! { AccountTile { account: AccountFace::All, pressed: Check::Off, unread: 0, onclick: |_| {} } },
    },
    // EdgeStrip.
    Case {
        component: "edge_strip",
        state: "default",
        make: || rsx! { EdgeStrip { onenter: |_| {} } },
    },
    // DragGhost, DropLine, Grip.
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
    Case {
        component: "drag_ghost",
        state: "grip",
        make: || rsx! { Grip { label: "Drag to move · click for options", onclick: |_| {} } },
    },
    // HoverStrip.
    Case {
        component: "hover_strip",
        state: "default",
        make: || rsx! { HoverStrip { actions: strip_actions() } },
    },
    // AppearancePicker.
    Case {
        component: "appearance_picker",
        state: "default",
        make: || rsx! { AppearancePicker { value: Appearance::default(), system: SystemPrefs::default(), onchange: |_| {} } },
    },
    Case {
        component: "appearance_picker",
        state: "system-dark-reduced",
        make: || {
            rsx! {
                AppearancePicker {
                    value: Appearance::default(),
                    system: SystemPrefs { scheme: Scheme::Dark, motion: ReducedMotion::Reduce, ..SystemPrefs::default() },
                    onchange: |_| {},
                }
            }
        },
    },
    Case {
        component: "appearance_picker",
        state: "dark-red-calm",
        make: || {
            rsx! {
                AppearancePicker {
                    value: Appearance { theme: Theme::Dark, accent: Accent::Red, motion: Motion::Reduced },
                    system: SystemPrefs::default(),
                    onchange: |_| {},
                }
            }
        },
    },
];
