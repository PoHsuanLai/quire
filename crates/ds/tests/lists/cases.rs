//! Every list, sidebar and Space-frame component in every state, as data: the table the golden
//! test walks. Goldens live in `tests/snapshots/lists/<component>/<state>.html`.

use crate::rows::strip_actions;
use dioxus::prelude::*;
use ds::{
    Accent, Appearance, AppearancePicker, Motion, ReducedMotion, RowState, Scheme, SystemPrefs,
    Theme,
};
use ds::{
    AccountFace, AccountTile, AvatarFace, AvatarShape, AvatarSize, AvatarTone, Colour, CommandPill,
    DragGhost, DropLine, EdgeStrip, Exit, Grip, Hex, HoverStrip, Icon, ImageSource, ItemKind,
    MarkProvider, MarkSize, MarkStyle, PersonHue, Point, Presence, Preview, ProviderMark, Px,
    SidebarItem,
};
use ds::{Check, DropState, Selection, Shortcut, ShortcutKey};

/// One component in one state.
pub struct Case {
    pub component: &'static str,
    pub state: &'static str,
    pub make: fn() -> Element,
}

pub const DANA: AvatarFace = AvatarFace {
    initial: 'D',
    size: AvatarSize::Size16,
    tone: AvatarTone::Person(PersonHue(212)),
    shape: AvatarShape::Square,
};

const VIOLET: Colour = Colour::Solid(Hex([0x5b, 0x4f, 0xc4]));

fn item(kind: ItemKind, here: Selection, presence: Presence, preview: Option<Preview>) -> Element {
    let onclose = match kind {
        ItemKind::Today { .. } => Some(EventHandler::new(|_| {})),
        ItemKind::Place { .. } | ItemKind::Pinned { .. } => None,
    };
    let label = match kind {
        ItemKind::Place { .. } => "Inbox",
        ItemKind::Pinned { .. } => "Dana Okafor",
        ItemKind::Today { .. } => "Re: UIDL stability across servers",
    };
    rsx! {
        SidebarItem {
            state: RowState { selection: here, ..RowState::default() },
            kind,
            label,
            count: Some(4),
            presence,
            preview,
            onclick: |_| {},
            onclose,
        }
    }
}

const INBOX: ItemKind = ItemKind::Place { icon: Icon::Inbox };

/// The Inbox place playing `drop` in a drag.
fn dropped_item(drop: DropState) -> Element {
    rsx! {
        SidebarItem {
            state: RowState { selection: Selection::Unselected, drop, ..RowState::default() },
            kind: INBOX,
            label: "Inbox",
            count: None,
            presence: Presence::Present,
            preview: None,
            onclick: |_| {},
            onclose: None,
        }
    }
}

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
    // SidebarItem: the three kinds, current with its seal, the previews.
    Case {
        component: "sidebar_item",
        state: "place",
        make: || item(INBOX, Selection::Unselected, Presence::Present, None),
    },
    Case {
        component: "sidebar_item",
        state: "place-current",
        make: || item(INBOX, Selection::Selected, Presence::Present, None),
    },
    Case {
        component: "sidebar_item",
        state: "place-destination",
        make: || {
            item(
                ItemKind::Place {
                    icon: Icon::Archive,
                },
                Selection::Unselected,
                Presence::Present,
                Some(Preview::Destination),
            )
        },
    },
    Case {
        component: "sidebar_item",
        state: "pinned",
        make: || {
            item(
                ItemKind::Pinned { avatar: DANA },
                Selection::Unselected,
                Presence::Present,
                None,
            )
        },
    },
    Case {
        component: "sidebar_item",
        state: "today-entering",
        make: || {
            item(
                ItemKind::Today { avatar: DANA },
                Selection::Unselected,
                Presence::Entering,
                None,
            )
        },
    },
    Case {
        component: "sidebar_item",
        state: "today-present",
        make: || {
            item(
                ItemKind::Today { avatar: DANA },
                Selection::Selected,
                Presence::Present,
                None,
            )
        },
    },
    Case {
        component: "sidebar_item",
        state: "today-leaving",
        make: || {
            item(
                ItemKind::Today { avatar: DANA },
                Selection::Unselected,
                Presence::Leaving(Exit::Row),
                None,
            )
        },
    },
    Case {
        component: "sidebar_item",
        state: "drop-target",
        make: || dropped_item(DropState::Target),
    },
    Case {
        component: "sidebar_item",
        state: "drag-source",
        make: || dropped_item(DropState::Source),
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
