//! Controls: every control in every state its props can express.

use super::external_icons::ExternalIcons;
use super::status_items::StatusItems;
use super::{Section, Specimen};
use dioxus::prelude::*;
use ds::{
    AccountFace, AccountTile, Anim, Availability, Avatar, AvatarFace, AvatarShape, AvatarSize,
    AvatarTone, Button, ButtonVariant, Chip, ChipVariant, Colour, CommandPill, Count, Fraction,
    HeaderKind, Hex, Icon, IconButton, IconButtonVariant, InputVariant, Kbd, KbdSize, Key,
    LabelHue, MarkSize, MarkStyle, PersonHue, Provider, ProviderMark, SearchField, SectionHeader,
    SegSize, SegmentedControl, Shortcut, Slider, Spinner, SpinnerKind, Switch, SyncHalo, SyncState,
    Tabs, TextInput, Toggle, Verdict, use_pulse,
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

const PROVIDERS: [Provider; 6] = [
    Provider::Google,
    Provider::Microsoft,
    Provider::Fastmail,
    Provider::ICloud,
    Provider::Yahoo,
    Provider::Imap,
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
        StatusItems {}
        ExternalIcons {}
        Choosers {}
        Fields {}
        Chips {}
        Faces {}
        Marks {}
    }
}

#[component]
fn Buttons() -> Element {
    rsx! {
        Section { title: "Button", note: "Five variants, each at rest, pressed off and on, and disabled; with and without an icon.",
            for (variant , name) in BUTTONS {
                div { class: "g-row",
                    span { class: "g-name g-type-name", "{name}" }
                    for (state , pressed , availability) in BUTTON_STATES {
                        Button { variant, label: state, pressed, availability, onclick: |_| {} }
                    }
                    Button { variant, label: "With icon", icon: Some(Icon::Archive), onclick: |_| {} }
                }
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
fn Fields() -> Element {
    let mut text = use_signal(String::new);
    let mut search = use_signal(|| "invoice".to_string());
    rsx! {
        Section { title: "TextInput and SearchField", note: "Boxed and inline; empty with a placeholder, filled, disabled. Type in the live ones.",
            div { class: "g-grid3",
                for variant in [InputVariant::Boxed, InputVariant::Inline] {
                    Specimen { name: "live",
                        TextInput { variant, label: "Live", value: text(), placeholder: "Type here", oninput: move |next| text.set(next) }
                    }
                    Specimen { name: "filled",
                        TextInput { variant, label: "Filled", value: "pohsuan@example.org", oninput: |_| {} }
                    }
                    Specimen { name: "disabled",
                        TextInput { variant, label: "Disabled", value: "", placeholder: "Not now", availability: Availability::Disabled, oninput: |_| {} }
                    }
                }
            }
            div { class: "g-grid2",
                Specimen { name: "search with tokens",
                    SearchField {
                        label: "Search",
                        value: search(),
                        placeholder: "Search mail",
                        tokens: vec!["from:dana".to_string(), "has:attachment".to_string()],
                        oninput: move |next| search.set(next),
                        onkey: |_| {},
                    }
                }
                Specimen { name: "search, empty",
                    SearchField { label: "Search", value: "", placeholder: "Search mail", tokens: Vec::new(), oninput: |_| {}, onkey: |_| {} }
                }
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
        Section { title: "ProviderMark and AccountTile", note: "Letters at tile, row and inline size; tiles pressed and not (the tile desaturates its colour when not pressed).",
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
            }
        }
    }
}
