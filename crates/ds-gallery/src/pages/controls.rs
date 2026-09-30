//! Controls: every control in every state its props can express.

use super::button_faces::ButtonFaces;
use super::dock_tiles::DockTiles;
use super::external_icons::ExternalIcons;
use super::fields::{FieldKinds, Fields};
use super::glyphs::Glyphs;
use super::plate_tints::PlateTints;
use super::status_items::StatusItems;
use super::{Section, Specimen};
use dioxus::prelude::*;
use ds::ControlSize;
use ds::Word;
use ds::{
    AccountFace, AccountTile, AddAccountTile, Availability, Avatar, AvatarFace, AvatarShape,
    AvatarSize, AvatarTone, Button, Check, Chip, ChipVariant, Colour, CommandPill, Fraction,
    HeaderKind, Hex, ImageSource, LabelHue, MarkProvider, MarkSize, MarkStyle, PersonHue, Progress,
    ProgressIndicator, ProgressStyle, ProviderMark, SectionHeader, SegmentedControl, Shortcut,
    ShortcutKey, Slider, Toggle, Verdict,
};
use ds::{Badge, BadgeContent, BadgeTone, KeyEquivalent, KeyStyle};
use ds::{Choice, Tracking};

const PROVIDERS: [MarkProvider; 7] = [
    MarkProvider::Google,
    MarkProvider::Microsoft,
    MarkProvider::Fastmail,
    MarkProvider::ICloud,
    MarkProvider::Yahoo,
    MarkProvider::Imap,
    MarkProvider::Local,
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
        super::controls_mailo6::MoreGlyphs {}
        super::controls_mailo6::PassThrough {}
        StatusItems {}
        ExternalIcons {}
        Glyphs {}
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
        Section { title: "CommandPill and Kbd",
            div { class: "g-row",
                CommandPill { label: "Search or run a command", shortcut: Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char('k')]), onclick: |_| {} }
                KeyEquivalent { shortcut: Shortcut(vec![ShortcutKey::Shift, ShortcutKey::Super, ShortcutKey::Char('p')]) , style: KeyStyle::Cap}
                KeyEquivalent { shortcut: Shortcut(vec![ShortcutKey::Super, ShortcutKey::Enter]), style: KeyStyle::Cap, size: ControlSize::Mini}
                KeyEquivalent { shortcut: Shortcut(vec![ShortcutKey::Escape, ShortcutKey::Tab, ShortcutKey::Backspace, ShortcutKey::Up, ShortcutKey::Down, ShortcutKey::Left, ShortcutKey::Right, ShortcutKey::Space]), style: KeyStyle::Cap, size: ControlSize::Mini}
            }
        }
    }
}

#[component]
fn Choosers() -> Element {
    let mut view = use_signal(|| 1u8);
    let mut on = use_signal(|| Check::On);
    let mut level = use_signal(|| Fraction(350));
    let mut tab = use_signal(|| 0u8);
    let mut count = use_signal(|| 3u32);
    // The spinner runs an operation started as the page opens (design/26 R4); the Details page
    // replays one.
    let busy = use_hook(|| ds::detail::Operation::Running(ds::detail::PendingToken::start()));
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
        Section { title: "SegmentedControl", note: "Live: click to change.",
            div { class: "g-row",
                SegmentedControl::<u8> { label: "View", choices: Choice::pairs(views.clone()), tracking: Tracking::SelectOne(view()), onchange: move |next| view.set(next) }
                SegmentedControl::<u8> { label: "View", choices: Choice::pairs(views), tracking: Tracking::SelectOne(view()), size: ControlSize::Mini, onchange: move |next| view.set(next) }
            }
            SegmentedControl::<u8> { label: "Mailbox", choices: Choice::pairs(tabs), tracking: Tracking::SelectOne(tab()), onchange: move |next| tab.set(next) }
        }
        Section { title: "Toggle and Slider",
            div { class: "g-row",
                Specimen { name: "live",
                    Toggle { label: "Live toggle", value: on(), onchange: move |next| on.set(next) }
                }
                Specimen { name: "off",
                    Toggle { label: "Off", value: Check::Off, onchange: |_| {} }
                }
                Specimen { name: "on, disabled",
                    Toggle { label: "On, disabled", value: Check::On, availability: Availability::Disabled, onchange: |_| {} }
                }
                Specimen { name: "off, disabled",
                    Toggle { label: "Off, disabled", value: Check::Off, availability: Availability::Disabled, onchange: |_| {} }
                }
                Specimen { name: "Mini (settings row), on", code: "26 x 15, knob 13".to_string(),
                    Toggle { label: "Mini on", value: Check::On, size: ds::ControlSize::Mini, onchange: |_| {} }
                }
                Specimen { name: "Small, on", code: "32 x 18, knob 16".to_string(),
                    Toggle { label: "Small on", value: Check::On, size: ds::ControlSize::Small, onchange: |_| {} }
                }
                Specimen { name: "Large, on", code: "38 x 22, knob 20".to_string(),
                    Toggle { label: "Large on", value: Check::On, size: ds::ControlSize::Large, onchange: |_| {} }
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
        Section { title: "Count, Spinner, SectionHeader", note: "Change the count to see it.",
            div { class: "g-row",
                Button { size: ControlSize::Mini, label: "+1", onclick: move |_| *count.write() += 1 }
                Button { size: ControlSize::Mini, label: "0", onclick: move |_| count.set(0) }
                Specimen { name: "item",
                    Badge { content: BadgeContent::Number(count()) , tone: BadgeTone::Quiet, size: ControlSize::Mini}
                }
                Specimen { name: "spin", ProgressIndicator { style: ProgressStyle::Spinner, progress: Progress::Unknown(busy), size: ControlSize::Small } }
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
    let person = AvatarFace {
        initial: 'D',
        size: AvatarSize::Size18,
        tone: AvatarTone::Person(PersonHue::of("dana@example.org")),
        shape: AvatarShape::Round,
    };
    rsx! {
        Section { title: "Chip", note: "Every variant; the person chip with its remove button.",
            div { class: "g-row",
                Chip { variant: ChipVariant::Accent, text: "Accent" }
                Chip { variant: ChipVariant::Neutral, text: "Neutral" }
                Chip { variant: ChipVariant::Token, text: "from:dana" }
                Chip { variant: ChipVariant::Status(Verdict::Pass), text: "4.8 : 1" }
                Chip { variant: ChipVariant::Status(Verdict::Fail), text: "2.1 : 1" }
                for hue in LabelHue::ALL.iter().copied() {
                    Chip { variant: ChipVariant::Label(hue), text: hue.slug() }
                }
            }
            div { class: "g-row",
                Chip { variant: ChipVariant::Person(person), text: "Dana Okafor", onremove: Some(EventHandler::new(|()| {})) }
                Chip { variant: ChipVariant::Person(person), text: "No remove" }
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
    let mut pressed = use_signal(|| Check::On);
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
                AccountTile { account: AccountFace::All, pressed: Check::On, unread: 12, onclick: |_| {} }
                AccountTile { account: one('P', MarkProvider::Google), pressed: pressed(), unread: 3, onclick: move |_| pressed.set(pressed().flipped()) }
                AccountTile { account: one('W', MarkProvider::Microsoft), pressed: Check::Off, unread: 0, onclick: |_| {} }
                AccountTile { account: one('G', MarkProvider::Google), pressed: Check::On, unread: 5, mark: MarkStyle::Image(favicon()), onclick: |_| {} }
                AccountTile { account: one('L', MarkProvider::Local), pressed: Check::On, unread: 1, onclick: |_| {} }
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
