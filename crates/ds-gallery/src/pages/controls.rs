//! Controls: every control in every state its props can express.

use super::button_faces::ButtonFaces;
use super::dock_tiles::DockTiles;
use super::external_icons::ExternalIcons;
use super::fields::{FieldKinds, Fields};
use super::plate_tints::PlateTints;
use super::status_items::StatusItems;
use super::{Section, Specimen};
use dioxus::prelude::*;
use ds::{
    AccountFace, AccountTile, AddAccountTile, Anim, Availability, Avatar, AvatarFace, AvatarShape,
    AvatarSize, AvatarTone, Button, ButtonVariant, Chip, ChipVariant, Colour, CommandPill, Count,
    Expanded, Fraction, HeaderKind, Hex, Icon, IconButton, IconButtonVariant, ImageSource, Kbd,
    KbdSize, Key, LabelHue, MarkSize, MarkStyle, PersonHue, Provider, ProviderMark, SectionHeader,
    SegSize, SegmentedControl, Shortcut, Slider, Spinner, SpinnerKind, Switch, SyncHalo, SyncState,
    Tabs, Toggle, Verdict, use_pulse,
};

const BUTTONS: [(ButtonVariant, &str); 5] = [
    (ButtonVariant::Primary, "Primary"),
    (ButtonVariant::Secondary, "Secondary"),
    (ButtonVariant::Mini, "Mini"),
    (ButtonVariant::Quiet, "Quiet"),
    (ButtonVariant::Danger, "Danger"),
];

const ICON_BUTTONS: [(IconButtonVariant, &str); 4] = [
    (IconButtonVariant::Tool, "Tool"),
    (IconButtonVariant::Foot, "Foot"),
    (IconButtonVariant::Strip, "Strip"),
    (IconButtonVariant::Pin, "Pin"),
];

/// The states a button's props can put it in.
const BUTTON_STATES: [(&str, Option<Switch>, Availability); 4] = [
    ("rest", None, Availability::Enabled),
    ("pressed off", Some(Switch::Off), Availability::Enabled),
    ("pressed on", Some(Switch::On), Availability::Enabled),
    ("disabled", None, Availability::Disabled),
];

const PROVIDERS: [Provider; 7] = [
    Provider::Google,
    Provider::Microsoft,
    Provider::Fastmail,
    Provider::ICloud,
    Provider::Yahoo,
    Provider::Imap,
    Provider::Local,
];

const AVATAR_SIZES: [AvatarSize; 8] = [
    AvatarSize::Size16,
    AvatarSize::Size18,
    AvatarSize::Size20,
    AvatarSize::Size22,
    AvatarSize::Size26,
    AvatarSize::Size28,
    AvatarSize::Size30,
    AvatarSize::Size34,
];

/// The controls page.
#[component]
pub fn ControlsPage() -> Element {
    rsx! {
        Buttons {}
        ButtonFaces {}
        super::controls_mailo5::ButtonsMailo5 {}
        StatusItems {}
        ExternalIcons {}
        DockTiles {}
        PlateTints {}
        Choosers {}
        Fields {}
        FieldKinds {}
        Chips {}
        Faces {}
        Marks {}
    }
}

#[component]
fn Buttons() -> Element {
    rsx! {
        Section { title: "Button", note: "Five variants, each at rest, pressed off and on, and disabled; with and without an icon. Named: a hover title and an assistive name over a terse label, and a trigger open and closed (aria-expanded).",
            for (variant , name) in BUTTONS {
                div { class: "g-row",
                    span { class: "g-name g-type-name", "{name}" }
                    for (state , pressed , availability) in BUTTON_STATES {
                        Button { variant, label: state, pressed, availability, onclick: |_| {} }
                    }
                    Button { variant, label: "With icon", icon: Some(Icon::Archive), onclick: |_| {} }
                }
            }
            div { class: "g-row",
                span { class: "g-name g-type-name", "Named" }
                Button { variant: ButtonVariant::Mini, label: "+", title: "Add account…", aria_label: "Add account", onclick: |_| {} }
                Button { variant: ButtonVariant::Quiet, label: "More", icon: Some(Icon::ChevronDown), expanded: Expanded::Open, onclick: |_| {} }
                Button { variant: ButtonVariant::Quiet, label: "More", icon: Some(Icon::ChevronDown), expanded: Expanded::Closed, onclick: |_| {} }
            }
        }
        Section { title: "IconButton", note: "Four variants; rest, pressed on, expanded, disabled.",
            div { class: "g-row",
                for (variant , name) in ICON_BUTTONS {
                    Specimen { name,
                        div { class: "g-row",
                            IconButton { variant, icon: Icon::Star, label: "{name} rest", onclick: |_| {} }
                            IconButton { variant, icon: Icon::Star, label: "{name} pressed", pressed: Some(Switch::On), onclick: |_| {} }
                            IconButton { variant, icon: Icon::ChevronDown, label: "{name} expanded", expanded: Some(Switch::On), onclick: |_| {} }
                            IconButton { variant, icon: Icon::Trash, label: "{name} disabled", availability: Availability::Disabled, onclick: |_| {} }
                        }
                    }
                }
            }
        }
        Section { title: "CommandPill and Kbd",
            div { class: "g-row",
                CommandPill { label: "Search or run a command", shortcut: Shortcut(vec![Key::Ctrl, Key::Char('k')]), onclick: |_| {} }
                Kbd { shortcut: Shortcut(vec![Key::Ctrl, Key::Shift, Key::Char('p')]) }
                Kbd { shortcut: Shortcut(vec![Key::Super, Key::Enter]), size: KbdSize::Small }
                Kbd { shortcut: Shortcut(vec![Key::Escape, Key::Tab, Key::Backspace, Key::Up, Key::Down, Key::Left, Key::Right, Key::Space]), size: KbdSize::Small }
            }
        }
    }
}

#[component]
fn Choosers() -> Element {
    let mut view = use_signal(|| 1u8);
    let mut on = use_signal(|| Switch::On);
    let mut level = use_signal(|| Fraction(350));
    let mut tab = use_signal(|| 0u8);
    let mut count = use_signal(|| 3u32);
    let views: Vec<(u8, String)> = ["List", "Columns", "Cards"]
        .into_iter()
        .zip(0..)
        .map(|(name, value)| (value, name.to_string()))
        .collect();
    let tabs: Vec<(u8, String)> = ["Inbox", "Starred", "Snoozed", "Sent"]
        .into_iter()
        .zip(0..)
        .map(|(name, value)| (value, name.to_string()))
        .collect();
    rsx! {
        Section { title: "SegmentedControl and Tabs", note: "Live: click to change.",
            div { class: "g-row",
                SegmentedControl::<u8> { label: "View", options: views.clone(), value: view(), onchange: move |next| view.set(next) }
                SegmentedControl::<u8> { label: "View", options: views, value: view(), size: SegSize::Small, onchange: move |next| view.set(next) }
            }
            Tabs::<u8> { label: "Mailbox", tabs, value: tab(), onchange: move |next| tab.set(next) }
        }
        Section { title: "Toggle and Slider",
            div { class: "g-row",
                Specimen { name: "live",
                    Toggle { label: "Live toggle", value: on(), onchange: move |next| on.set(next) }
                }
                Specimen { name: "off",
                    Toggle { label: "Off", value: Switch::Off, onchange: |_| {} }
                }
                Specimen { name: "on, disabled",
                    Toggle { label: "On, disabled", value: Switch::On, availability: Availability::Disabled, onchange: |_| {} }
                }
                Specimen { name: "off, disabled",
                    Toggle { label: "Off, disabled", value: Switch::Off, availability: Availability::Disabled, onchange: |_| {} }
                }
            }
            div { class: "g-grid4",
                Specimen { name: "live", code: format!("{} / 1000", level().0),
                    Slider { label: "Live slider", value: level(), step: Fraction(50), onchange: move |next| level.set(next) }
                }
                Specimen { name: "empty",
                    Slider { label: "Empty", value: Fraction(0), onchange: |_| {} }
                }
                Specimen { name: "full",
                    Slider { label: "Full", value: Fraction(1000), onchange: |_| {} }
                }
                Specimen { name: "disabled",
                    Slider { label: "Disabled", value: Fraction(600), availability: Availability::Disabled, onchange: |_| {} }
                }
            }
        }
        Section { title: "Count, Spinner, SyncHalo, SectionHeader", note: "Change the count to see its bump.",
            div { class: "g-row",
                Button { variant: ButtonVariant::Mini, label: "+1", onclick: move |_| *count.write() += 1 }
                Button { variant: ButtonVariant::Mini, label: "0", onclick: move |_| count.set(0) }
                Specimen { name: "item",
                    Count { value: count() }
                }
                Specimen { name: "spin", Spinner { kind: SpinnerKind::Spin } }
                Specimen { name: "breathe", Spinner { kind: SpinnerKind::Breathe } }
                Specimen { name: "halo idle",
                    SyncHalo { initial: 'P', tone: AvatarTone::Ink, state: SyncState::Idle }
                }
                Specimen { name: "halo busy",
                    SyncHalo { initial: 'P', tone: AvatarTone::Ink, state: SyncState::Busy }
                }
            }
            div { class: "g-grid4",
                SectionHeader { kind: HeaderKind::Frame, text: "Frame" }
                SectionHeader { kind: HeaderKind::Group, text: "Group", value: Some("12".to_string()) }
                SectionHeader { kind: HeaderKind::Field, text: "Field", value: Some("Light".to_string()) }
                SectionHeader { kind: HeaderKind::Menu, text: "Menu", action: Some(("Clear".to_string(), EventHandler::new(|()| {}))) }
            }
        }
    }
}

#[component]
fn Chips() -> Element {
    let flash = use_pulse(Anim::ChipFlash);
    let person = AvatarFace {
        initial: 'D',
        size: AvatarSize::Size18,
        tone: AvatarTone::Person(PersonHue::of("dana@example.org")),
        shape: AvatarShape::Round,
    };
    rsx! {
        Section { title: "Chip", note: "Every variant; the person chip with its remove button, and its flash ring on demand.",
            div { class: "g-row",
                Chip { variant: ChipVariant::Accent, text: "Accent" }
                Chip { variant: ChipVariant::Neutral, text: "Neutral" }
                Chip { variant: ChipVariant::Token, text: "from:dana" }
                Chip { variant: ChipVariant::Status(Verdict::Pass), text: "4.8 : 1" }
                Chip { variant: ChipVariant::Status(Verdict::Fail), text: "2.1 : 1" }
                for hue in LabelHue::ALL {
                    Chip { variant: ChipVariant::Label(hue), text: hue.slug() }
                }
            }
            div { class: "g-row",
                Chip { variant: ChipVariant::Person(person), text: "Dana Okafor", onremove: Some(EventHandler::new(|()| {})), pulse: Some(flash.key()) }
                Chip { variant: ChipVariant::Person(person), text: "No remove" }
                Button { variant: ButtonVariant::Mini, label: "Flash the person chip", onclick: move |_| flash.fire() }
            }
        }
    }
}

#[component]
fn Faces() -> Element {
    let tones = [
        ("ink", AvatarTone::Ink),
        ("stack", AvatarTone::Stack),
        (
            "account",
            AvatarTone::Account(Colour::Solid(Hex([0x2a, 0x5d, 0xb0]))),
        ),
        (
            "person",
            AvatarTone::Person(PersonHue::of("sam@example.org")),
        ),
    ];
    rsx! {
        Section { title: "Avatar", note: "Every size, each tone, round and square.",
            for (name , tone) in tones {
                div { class: "g-row",
                    span { class: "g-name g-type-name", "{name}" }
                    for size in AVATAR_SIZES {
                        Avatar { initial: 'Q', size, tone }
                    }
                    Avatar { initial: 'Q', size: AvatarSize::Size28, tone, shape: AvatarShape::Square }
                }
            }
        }
    }
}

#[component]
fn Marks() -> Element {
    let mut pressed = use_signal(|| Switch::On);
    let one = |initial, provider| AccountFace::One {
        initial,
        colour: Colour::Solid(Hex([0x1a, 0x73, 0xe8])),
        provider,
        address: Some(format!("{initial}@example.org").to_lowercase()),
    };
    rsx! {
        Section { title: "ProviderMark and AccountTile", note: "Letters at tile, row and inline size; tiles pressed and not (the tile desaturates its colour when not pressed), one showing the favicon the app supplies (mark: MarkStyle::Image), a local-folders account (Provider::Local: the neutral folder), and the Add account tile after them.",
            for size in [MarkSize::Tile, MarkSize::Row, MarkSize::Inline] {
                div { class: "g-row",
                    for provider in PROVIDERS {
                        ProviderMark { provider, size, style: MarkStyle::Letter }
                    }
                }
            }
            div { class: "g-row",
                AccountTile { account: AccountFace::All, pressed: Switch::On, unread: 12, onclick: |_| {} }
                AccountTile { account: one('P', Provider::Google), pressed: pressed(), unread: 3, onclick: move |_| pressed.set(match pressed() { Switch::On => Switch::Off, Switch::Off => Switch::On }) }
                AccountTile { account: one('W', Provider::Microsoft), pressed: Switch::Off, unread: 0, onclick: |_| {} }
                AccountTile { account: one('G', Provider::Google), pressed: Switch::On, unread: 5, mark: MarkStyle::Image(favicon()), onclick: |_| {} }
                AccountTile { account: one('L', Provider::Local), pressed: Switch::On, unread: 1, onclick: |_| {} }
                AddAccountTile { title: "Add account…", onclick: |_| {} }
            }
        }
    }
}

/// A stand-in favicon, as an app would supply one: a data URI quire never fetches.
fn favicon() -> ImageSource {
    let svg = "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'><circle cx='8' cy='8' r='7' fill='#1A73E8'/><circle cx='8' cy='8' r='3' fill='#FFFFFF'/></svg>";
    ImageSource(format!(
        "data:image/svg+xml;base64,{}",
        crate::data_uri::base64(svg.as_bytes())
    ))
}
