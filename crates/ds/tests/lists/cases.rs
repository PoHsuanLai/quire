//! Every list, sidebar and Space-frame component in every state, as data: the table the golden
//! test walks. Goldens live in `tests/snapshots/lists/<component>/<state>.html`.

use crate::rows::strip_actions;
use dioxus::prelude::*;
use ds::components::vocab::{DropState, Here, Key, PulseKey, Shortcut, Switch};
use ds::{
    Accent, Appearance, AppearancePicker, CardAccent, DotIndex, FrameVars, Grain, Motion, PRESETS,
    ReducedMotion, Scheme, SpaceDot, SpaceEditor, SpaceLook, SystemPrefs, Theme,
};
use ds::{
    AccountFace, AccountTile, Anim, AvatarFace, AvatarShape, AvatarSize, AvatarTone, Colour,
    CommandPill, DragGhost, DropLine, EdgeStrip, Exit, Grip, Hex, HoverStrip, Icon, ImageSource,
    ItemKind, MarkSize, MarkStyle, PersonHue, Point, Presence, Preview, Provider, ProviderMark, Px,
    SidebarItem, SyncHalo, SyncState,
};

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

fn item(
    kind: ItemKind,
    here: Here,
    presence: Presence,
    preview: Option<Preview>,
    pulse: PulseKey,
) -> Element {
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
            kind,
            label,
            here,
            count: Some(4),
            presence,
            preview,
            pulse,
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
            kind: INBOX,
            label: "Inbox",
            here: Here::Elsewhere,
            count: None,
            presence: Presence::Present,
            preview: None,
            pulse: GULP(),
            onclick: |_| {},
            onclose: None,
            drop,
        }
    }
}
const GULP: fn() -> PulseKey = || PulseKey::rest(Anim::Gulp);

fn mark(provider: Provider, size: MarkSize) -> Element {
    rsx! { ProviderMark { provider, size, style: MarkStyle::Letter } }
}

pub const CASES: &[Case] = &[
    // CommandPill.
    Case {
        component: "command_pill",
        state: "default",
        make: || rsx! { CommandPill { label: "Search or run a command", shortcut: Shortcut(vec![Key::Ctrl, Key::Char('t')]), onclick: |_| {} } },
    },
    // ProviderMark: every provider's letter across the three sizes, and a favicon.
    Case {
        component: "provider_mark",
        state: "google-tile",
        make: || mark(Provider::Google, MarkSize::Tile),
    },
    Case {
        component: "provider_mark",
        state: "microsoft-row",
        make: || mark(Provider::Microsoft, MarkSize::Row),
    },
    Case {
        component: "provider_mark",
        state: "fastmail-inline",
        make: || mark(Provider::Fastmail, MarkSize::Inline),
    },
    Case {
        component: "provider_mark",
        state: "icloud-inline",
        make: || mark(Provider::ICloud, MarkSize::Inline),
    },
    Case {
        component: "provider_mark",
        state: "yahoo-row",
        make: || mark(Provider::Yahoo, MarkSize::Row),
    },
    Case {
        component: "provider_mark",
        state: "imap-tile",
        make: || mark(Provider::Imap, MarkSize::Tile),
    },
    Case {
        component: "provider_mark",
        state: "image",
        make: || rsx! { ProviderMark { provider: Provider::Google, size: MarkSize::Tile, style: MarkStyle::Image(ImageSource("data:image/png;base64,iVBORw0KGgo=".to_string())) } },
    },
    // AccountTile: All, one pressed, one unpressed (desaturated), nothing unread.
    Case {
        component: "account_tile",
        state: "all-pressed",
        make: || rsx! { AccountTile { account: AccountFace::All, pressed: Switch::On, unread: 4, onclick: |_| {} } },
    },
    Case {
        component: "account_tile",
        state: "one-pressed",
        make: || rsx! { AccountTile { account: AccountFace::One { initial: 'P', colour: VIOLET, provider: Provider::Google, address: Some("poh@acme.example".to_string()) }, pressed: Switch::On, unread: 2, onclick: |_| {} } },
    },
    Case {
        component: "account_tile",
        state: "one-unpressed",
        make: || rsx! { AccountTile { account: AccountFace::One { initial: 'P', colour: VIOLET, provider: Provider::Fastmail, address: None }, pressed: Switch::Off, unread: 2, onclick: |_| {} } },
    },
    Case {
        component: "account_tile",
        state: "none-unread",
        make: || rsx! { AccountTile { account: AccountFace::All, pressed: Switch::Off, unread: 0, onclick: |_| {} } },
    },
    // SyncHalo.
    Case {
        component: "sync_halo",
        state: "idle",
        make: || rsx! { SyncHalo { initial: 'P', tone: AvatarTone::Ink, state: SyncState::Idle } },
    },
    Case {
        component: "sync_halo",
        state: "busy",
        make: || rsx! { SyncHalo { initial: 'P', tone: AvatarTone::Account(VIOLET), state: SyncState::Busy } },
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
    // SidebarItem: the three kinds, current with its seal, the previews and pulses.
    Case {
        component: "sidebar_item",
        state: "place",
        make: || item(INBOX, Here::Elsewhere, Presence::Present, None, GULP()),
    },
    Case {
        component: "sidebar_item",
        state: "place-current",
        make: || item(INBOX, Here::Current, Presence::Present, None, GULP()),
    },
    Case {
        component: "sidebar_item",
        state: "place-destination",
        make: || {
            item(
                ItemKind::Place {
                    icon: Icon::Archive,
                },
                Here::Elsewhere,
                Presence::Present,
                Some(Preview::Destination),
                GULP(),
            )
        },
    },
    Case {
        component: "sidebar_item",
        state: "place-gulp-a",
        make: || {
            item(
                INBOX,
                Here::Elsewhere,
                Presence::Present,
                None,
                GULP().fired(),
            )
        },
    },
    Case {
        component: "sidebar_item",
        state: "place-gulp-b",
        make: || {
            item(
                INBOX,
                Here::Elsewhere,
                Presence::Present,
                None,
                GULP().fired().fired(),
            )
        },
    },
    Case {
        component: "sidebar_item",
        state: "pinned",
        make: || {
            item(
                ItemKind::Pinned { avatar: DANA },
                Here::Elsewhere,
                Presence::Present,
                None,
                GULP(),
            )
        },
    },
    Case {
        component: "sidebar_item",
        state: "today-entering",
        make: || {
            item(
                ItemKind::Today { avatar: DANA },
                Here::Elsewhere,
                Presence::Entering,
                None,
                GULP(),
            )
        },
    },
    Case {
        component: "sidebar_item",
        state: "today-present",
        make: || {
            item(
                ItemKind::Today { avatar: DANA },
                Here::Current,
                Presence::Present,
                None,
                GULP(),
            )
        },
    },
    Case {
        component: "sidebar_item",
        state: "today-leaving",
        make: || {
            item(
                ItemKind::Today { avatar: DANA },
                Here::Elsewhere,
                Presence::Leaving(Exit::TabOut),
                None,
                GULP(),
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
                    value: Appearance { theme: Theme::Dark, accent: Accent::Red, motion: Motion::Calm },
                    system: SystemPrefs::default(),
                    onchange: |_| {},
                }
            }
        },
    },
    // SpaceEditor: the two sample Spaces in each scheme and a one-dot Postmark Space.
    Case {
        component: "space_editor",
        state: "work-light",
        make: || editor(preset_look(0, Grain(35)), Scheme::Light, 0),
    },
    Case {
        component: "space_editor",
        state: "work-dark",
        make: || editor(preset_look(0, Grain(35)), Scheme::Dark, 1),
    },
    Case {
        component: "space_editor",
        state: "home-light",
        make: || editor(preset_look(1, Grain(55)), Scheme::Light, 2),
    },
    Case {
        component: "space_editor",
        state: "one-dot-postmark",
        make: || {
            editor(
                SpaceLook {
                    card_accent: CardAccent::Postmark,
                    ..preset_look(2, Grain(40))
                },
                Scheme::Light,
                0,
            )
        },
    },
    Case {
        component: "space_editor",
        state: "named",
        make: || rsx! { SpaceEditor { look: preset_look(0, Grain(35)), scheme: Scheme::Light, active_dot: DotIndex(0), name: "Work".to_string(), onchange: |_| {}, on_active_dot: |_| {} } },
    },
    // SpaceDot.
    Case {
        component: "space_editor",
        state: "space-dot-current",
        make: || rsx! { SpaceDot { name: "Work", frame: FrameVars::of(&preset_look(0, Grain(35)), Scheme::Light), here: Here::Current, shortcut: Shortcut(vec![Key::Ctrl, Key::Char('1')]), onclick: |_| {} } },
    },
    Case {
        component: "space_editor",
        state: "space-dot-elsewhere",
        make: || rsx! { SpaceDot { name: "Home", frame: FrameVars::of(&preset_look(1, Grain(55)), Scheme::Dark), here: Here::Elsewhere, shortcut: Shortcut(vec![Key::Ctrl, Key::Char('2')]), onclick: |_| {} } },
    },
];

/// Preset `index` as a Space that lends its hue to the card.
pub fn preset_look(index: usize, grain: Grain) -> SpaceLook {
    SpaceLook {
        dots: PRESETS[index].dots.to_vec(),
        grain,
        theme: Theme::System,
        card_accent: CardAccent::SpaceHue,
    }
}

pub fn editor(look: SpaceLook, scheme: Scheme, active: u8) -> Element {
    rsx! { SpaceEditor { look, scheme, active_dot: DotIndex(active), onchange: |_| {} } }
}
