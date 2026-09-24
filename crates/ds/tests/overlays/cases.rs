//! Every overlay component in every state, as data: the table the golden test walks. Each case
//! renders inside a `Ds`, so floating surfaces arrive through its overlay host; `wait` lets the
//! case's timers run first (a hover card's 450 ms intent, an entrance settling).

use dioxus::prelude::*;
use ds::components::vocab::{Check, Fraction, Key, Shortcut, Switch};
use ds::{Align, Availability, Button, ButtonVariant};
use ds::{
    Anchor, AvatarFace, AvatarShape, AvatarSize, AvatarTone, BubbleAction, BubbleButton,
    BubbleMode, CommandPalette, Dismiss, Elevation, Filter, FlagTone, Glyph, HoverCard,
    HoverCardPart, HoverEvent, HoverKey, HoverKind, HoverMessage, HoverStat, HoverTarget, Icon,
    KeyHint, LinkPill, LinkTarget, Menu, MenuEntrance, MenuEntry, MenuKind, Peek, PeekMode,
    PersonHue, Placement, Point, Popover, Px, Rect, Scrim, SelectionBubble, SendPhase, SendPill,
    Sheet, Side, Size, Tile, Tooltip, TooltipKind, Trail, UndoToken, use_hover_hub, use_toasts,
};
use std::time::Duration;

/// One component in one state.
pub struct Case {
    pub component: &'static str,
    pub state: &'static str,
    pub make: fn() -> Element,
    pub wait: Duration,
}

const NOW: Duration = Duration::ZERO;
/// Past the 450 ms hover intent.
const INTENT: Duration = Duration::from_millis(520);
/// Past every entrance's settle at Standard (`settle(PeekIn)` = 454 ms).
const SETTLED: Duration = Duration::from_millis(520);
/// Past the one-frame wait before the send pill and the toast rise.
const FRAME: Duration = Duration::from_millis(80);

const DANA: AvatarFace = AvatarFace {
    initial: 'D',
    size: AvatarSize::Size20,
    tone: AvatarTone::Person(PersonHue(212)),
    shape: AvatarShape::Round,
};

/// A button's rect: 100 across, 40 down, 60 x 24.
fn button_rect() -> Rect {
    Rect {
        origin: Point {
            x: Px(100.0),
            y: Px(40.0),
        },
        size: Size {
            width: Px(60.0),
            height: Px(24.0),
        },
    }
}

fn item(value: u8, title: &str) -> MenuEntry<u8> {
    MenuEntry::Item {
        availability: Availability::Enabled,
        value,
        title: title.to_string(),
        detail: None,
        tile: None,
        trail: Trail::None,
        check: None,
    }
}

/// A bar status menu (bar gaps): two status lines, a rule, one item.
fn status_lines() -> Vec<MenuEntry<u8>> {
    vec![
        MenuEntry::Info {
            title: "Wired: connected".to_string(),
            detail: Some("192.168.1.4".to_string()),
        },
        MenuEntry::Info {
            title: "Battery 82%".to_string(),
            detail: None,
        },
        MenuEntry::Separator,
        item(1, "Network settings…"),
    ]
}

/// The snooze menu: a header, three timed items with tiles and trails, a rule, one more.
fn snooze() -> Vec<MenuEntry<u8>> {
    vec![
        MenuEntry::Header("Snooze until".to_string()),
        MenuEntry::Item {
            availability: Availability::Enabled,
            value: 1,
            title: "Later today".to_string(),
            detail: Some("18:00".to_string()),
            tile: Some(Tile::Icon(Icon::Clock)),
            trail: Trail::Shortcut(Shortcut(vec![Key::Ctrl, Key::Char('l')])),
            check: None,
        },
        MenuEntry::Item {
            availability: Availability::Enabled,
            value: 2,
            title: "Tomorrow".to_string(),
            detail: Some("08:00".to_string()),
            tile: Some(Tile::Text("T".to_string())),
            trail: Trail::Note("Thu".to_string()),
            check: Some(Check::Checked),
        },
        MenuEntry::Item {
            availability: Availability::Enabled,
            value: 3,
            title: "Dana Okafor".to_string(),
            detail: None,
            tile: Some(Tile::Avatar(DANA)),
            trail: Trail::None,
            check: Some(Check::Unchecked),
        },
        MenuEntry::Separator,
        item(4, "Pick a date…"),
    ]
}

fn group_by() -> Vec<MenuEntry<u8>> {
    vec![
        MenuEntry::Item {
            availability: Availability::Enabled,
            value: 1,
            title: "Date".to_string(),
            detail: None,
            tile: None,
            trail: Trail::Note("D".to_string()),
            check: Some(Check::Checked),
        },
        MenuEntry::Item {
            availability: Availability::Enabled,
            value: 2,
            title: "Sender".to_string(),
            detail: None,
            tile: None,
            trail: Trail::None,
            check: Some(Check::Unchecked),
        },
    ]
}

/// A disabled item, a submenu parent and a disabled submenu parent (quire gap Q7).
fn nested() -> Vec<MenuEntry<u8>> {
    let off = |value: u8, title: &str| MenuEntry::Item {
        availability: Availability::Disabled,
        value,
        title: title.to_string(),
        detail: None,
        tile: None,
        trail: Trail::None,
        check: None,
    };
    vec![
        item(1, "Open"),
        off(2, "Pause"),
        MenuEntry::Submenu {
            title: "More".to_string(),
            tile: Some(Tile::Icon(Icon::Settings)),
            availability: Availability::Enabled,
            children: vec![item(10, "About")],
        },
        MenuEntry::Submenu {
            title: "Services".to_string(),
            tile: None,
            availability: Availability::Disabled,
            children: vec![item(20, "Restart")],
        },
    ]
}

fn palette_groups() -> Vec<(String, Vec<MenuEntry<u8>>)> {
    vec![
        (
            "Top hit".to_string(),
            vec![MenuEntry::Item {
                availability: Availability::Enabled,
                value: 1,
                title: "Re: UIDL stability".to_string(),
                detail: Some("Treat UIDL as a hint, not an identity".to_string()),
                tile: Some(Tile::Avatar(AvatarFace {
                    size: AvatarSize::Size34,
                    ..DANA
                })),
                trail: Trail::Shortcut(Shortcut(vec![Key::Ctrl, Key::Char('1')])),
                check: None,
            }],
        ),
        ("People".to_string(), Vec::new()),
        (
            "Actions".to_string(),
            vec![MenuEntry::Item {
                availability: Availability::Enabled,
                value: 2,
                title: "Sync now".to_string(),
                detail: None,
                tile: Some(Tile::Icon(Icon::Refresh)),
                trail: Trail::None,
                check: None,
            }],
        ),
    ]
}

/// A sender target with its card, the card open once the hub says so.
#[component]
fn SenderCard(kind: HoverKind) -> Element {
    let hub = use_hover_hub();
    let key = HoverKey("sender:3".to_string());
    use_hook({
        let key = key.clone();
        move || hub.feed(HoverEvent::Over((key, kind)))
    });
    let open = hub.open().or(hub.leaving());
    rsx! {
        HoverTarget { hover_key: key, kind, "Dana Okafor" }
        if let Some((open, _)) = open {
            HoverCard { key: "{open.0}", kind,
                div { class: "ds-hovercard-person",
                    ds::Avatar { initial: 'D', size: AvatarSize::Size34, tone: AvatarTone::Ink }
                    div {
                        h5 { class: "ds-hovercard-title", "Dana Okafor" }
                        div { class: "ds-hovercard-sub", "dana@example.com" }
                    }
                }
                div { class: "ds-hovercard-stats",
                    span { b { "14" } "threads" }
                    span { b { "Mon" } "last wrote" }
                }
                div { class: "ds-hovercard-flag", "data-tone": "danger",
                    Glyph { icon: Icon::OctagonAlert, size: ds::IconSize::Compact }
                    span { "Not the address Dana usually writes from." }
                }
                div { class: "ds-hovercard-foot",
                    "stays unread while you look"
                    span { class: "ds-hovercard-keys", ds::Kbd { shortcut: Shortcut(vec![Key::Space]) } " peek" }
                }
                div { class: "ds-hovercard-actions",
                    Button { variant: ButtonVariant::Mini, label: "Reply", onclick: |_| {} }
                }
            }
        }
    }
}

/// A sender card composed from parts, open: the same card [`SenderCard`] writes by hand.
#[component]
fn PartsCard(parts: Vec<HoverCardPart>) -> Element {
    let hub = use_hover_hub();
    let key = HoverKey("sender:3".to_string());
    use_hook({
        let key = key.clone();
        move || hub.feed(HoverEvent::Over((key, HoverKind::Sender)))
    });
    let open = hub.open().or(hub.leaving());
    rsx! {
        HoverTarget { hover_key: key, kind: HoverKind::Sender, "Dana Okafor" }
        if let Some((open, _)) = open {
            HoverCard { key: "{open.0}", kind: HoverKind::Sender, parts: parts.clone() }
        }
    }
}

fn person() -> HoverCardPart {
    HoverCardPart::Person {
        initial: 'D',
        tone: AvatarTone::Ink,
        title: "Dana Okafor".to_string(),
        sub: Some("dana@example.com".to_string()),
    }
}

fn stats() -> HoverCardPart {
    HoverCardPart::Stats(vec![
        HoverStat {
            value: "14".to_string(),
            label: "threads".to_string(),
        },
        HoverStat {
            value: "Mon".to_string(),
            label: "last wrote".to_string(),
        },
    ])
}

fn flag(tone: FlagTone) -> HoverCardPart {
    HoverCardPart::Flag {
        tone,
        icon: Icon::OctagonAlert,
        text: "Not the address Dana usually writes from.".to_string(),
    }
}

fn foot() -> HoverCardPart {
    HoverCardPart::Foot {
        text: "stays unread while you look".to_string(),
        keys: Some(KeyHint {
            shortcut: Shortcut(vec![Key::Space]),
            label: "peek".to_string(),
        }),
    }
}

fn actions() -> HoverCardPart {
    HoverCardPart::Actions(vec![rsx! {
        Button { variant: ButtonVariant::Mini, label: "Reply", onclick: |_| {} }
    }])
}

fn messages() -> HoverCardPart {
    HoverCardPart::Messages(vec![
        HoverMessage {
            initial: 'D',
            tone: AvatarTone::Person(PersonHue(212)),
            name: "Dana".to_string(),
            text: "The UIDL list is stable across reconnects now.".to_string(),
        },
        HoverMessage {
            initial: 'M',
            tone: AvatarTone::Ink,
            name: "Me".to_string(),
            text: "Great, merging.".to_string(),
        },
    ])
}

/// The toast after an operation pushed it.
#[component]
fn Pushed(undo: Option<UndoToken>) -> Element {
    let toasts = use_toasts();
    use_hook(move || toasts.push("Archived".to_string(), undo));
    rsx! {}
}

fn bubble_actions() -> Vec<BubbleAction> {
    vec![
        BubbleAction::Button(BubbleButton {
            label: rsx! { b { "B" } },
            title: "Bold (Ctrl B)".to_string(),
            pressed: Some(Switch::On),
            onclick: EventHandler::new(|()| {}),
        }),
        BubbleAction::Button(BubbleButton {
            label: rsx! { i { "i" } },
            title: "Italic (Ctrl I)".to_string(),
            pressed: Some(Switch::Off),
            onclick: EventHandler::new(|()| {}),
        }),
        BubbleAction::Button(BubbleButton {
            label: rsx! { "</>" },
            title: "Inline code (Ctrl E)".to_string(),
            pressed: None,
            onclick: EventHandler::new(|()| {}),
        }),
        BubbleAction::Separator,
        BubbleAction::Button(BubbleButton {
            label: rsx! {
                Glyph { icon: Icon::Link, size: ds::IconSize::Small }
                "Link"
            },
            title: "Link (Ctrl K)".to_string(),
            pressed: None,
            onclick: EventHandler::new(|()| {}),
        }),
    ]
}

fn selection() -> Rect {
    Rect {
        origin: Point {
            x: Px(300.0),
            y: Px(200.0),
        },
        size: Size {
            width: Px(120.0),
            height: Px(18.0),
        },
    }
}

pub const CASES: &[Case] = &[
    // Menu: the four kinds, an empty list, and settled after its entrance.
    Case {
        component: "menu",
        state: "rich",
        make: || rsx! { Menu { kind: MenuKind::Rich, anchor: Anchor::Rect(button_rect()), entries: snooze(), onpick: |_| {}, onclose: |_| {} } },
        wait: NOW,
    },
    Case {
        component: "menu",
        state: "rich-present",
        make: || rsx! { Menu { kind: MenuKind::Rich, anchor: Anchor::Rect(button_rect()), entries: snooze(), onpick: |_| {}, onclose: |_| {} } },
        wait: SETTLED,
    },
    Case {
        component: "menu",
        state: "slim-typing",
        make: || rsx! { Menu { kind: MenuKind::Slim, anchor: Anchor::Rect(button_rect()), entries: snooze(), filter: Filter::Typing, onpick: |_| {}, onclose: |_| {} } },
        wait: NOW,
    },
    Case {
        component: "menu",
        state: "dropdown",
        make: || rsx! { Menu { kind: MenuKind::Dropdown, anchor: Anchor::Rect(button_rect()), entries: group_by(), onpick: |_| {}, onclose: |_| {} } },
        wait: NOW,
    },
    Case {
        component: "menu",
        state: "context",
        make: || rsx! { Menu { kind: MenuKind::Context, anchor: Anchor::Point(Point { x: Px(420.0), y: Px(310.0) }), entries: vec![item(1, "Archive"), item(2, "Snooze…")], onpick: |_| {}, onclose: |_| {} } },
        wait: NOW,
    },
    Case {
        component: "menu",
        state: "empty",
        make: || rsx! { Menu::<u8> { kind: MenuKind::Slim, anchor: Anchor::Rect(button_rect()), entries: Vec::new(), onpick: |_| {}, onclose: |_| {} } },
        wait: NOW,
    },
    Case {
        component: "menu",
        state: "disabled-and-submenu",
        make: || rsx! { Menu { kind: MenuKind::Context, anchor: Anchor::Point(Point { x: Px(40.0), y: Px(40.0) }), entries: nested(), onpick: |_| {}, onclose: |_| {} } },
        wait: NOW,
    },
    Case {
        component: "menu",
        state: "dropdown-disabled-and-submenu",
        make: || rsx! { Menu { kind: MenuKind::Dropdown, anchor: Anchor::Rect(button_rect()), entries: nested(), onpick: |_| {}, onclose: |_| {} } },
        wait: NOW,
    },
    Case {
        component: "menu",
        state: "bar-status-lines",
        make: || rsx! { Menu { kind: MenuKind::Dropdown, anchor: Anchor::Rect(button_rect()), entries: status_lines(), entrance: MenuEntrance::Instant, onpick: |_| {}, onclose: |_| {} } },
        wait: NOW,
    },
    // Popover: one per elevation and dismissal.
    Case {
        component: "popover",
        state: "pop",
        make: || rsx! { Popover { anchor: Anchor::Rect(button_rect()), placement: Placement::new(Side::Bottom, Align::Start), gap: Px(6.0), onclose: |_| {}, "Anything" } },
        wait: NOW,
    },
    Case {
        component: "popover",
        state: "bubble-esc-only",
        make: || rsx! { Popover { anchor: Anchor::Rect(button_rect()), placement: Placement::new(Side::Top, Align::Center), gap: Px(8.0), elevation: Elevation::Bubble, dismiss: Dismiss::EscOnly, onclose: |_| {}, "Anything" } },
        wait: NOW,
    },
    Case {
        component: "popover",
        state: "sheet-owner-closes",
        make: || rsx! { Popover { anchor: Anchor::Point(Point { x: Px(40.0), y: Px(40.0) }), placement: Placement::new(Side::Right, Align::End), gap: Px(0.0), elevation: Elevation::Sheet, dismiss: Dismiss::None, onclose: |_| {}, "Anything" } },
        wait: NOW,
    },
    // HoverCard: the target at rest, then each kind of card open after the intent.
    Case {
        component: "hover_card",
        state: "target",
        make: || rsx! { HoverTarget { hover_key: HoverKey("thread:88".to_string()), kind: HoverKind::Thread, "Re: UIDL stability" } },
        wait: NOW,
    },
    Case {
        component: "hover_card",
        state: "pending",
        make: || rsx! { SenderCard { kind: HoverKind::Sender } },
        wait: NOW,
    },
    Case {
        component: "hover_card",
        state: "sender-open",
        make: || rsx! { SenderCard { kind: HoverKind::Sender } },
        wait: INTENT,
    },
    Case {
        component: "hover_card",
        state: "side-open",
        make: || rsx! { SenderCard { kind: HoverKind::Side } },
        wait: INTENT,
    },
    // HoverCard parts: each block alone, and the sender card composed from parts (its golden
    // equals `sender-open`, the same card written by hand).
    Case {
        component: "hover_card",
        state: "part-title",
        make: || rsx! { PartsCard { parts: vec![HoverCardPart::Title("Re: UIDL stability".to_string())] } },
        wait: INTENT,
    },
    Case {
        component: "hover_card",
        state: "part-sub",
        make: || rsx! { PartsCard { parts: vec![HoverCardPart::Sub("3 messages · Mon".to_string())] } },
        wait: INTENT,
    },
    Case {
        component: "hover_card",
        state: "part-person",
        make: || rsx! { PartsCard { parts: vec![person()] } },
        wait: INTENT,
    },
    Case {
        component: "hover_card",
        state: "part-stats",
        make: || rsx! { PartsCard { parts: vec![stats()] } },
        wait: INTENT,
    },
    Case {
        component: "hover_card",
        state: "part-flag-danger",
        make: || rsx! { PartsCard { parts: vec![flag(FlagTone::Danger)] } },
        wait: INTENT,
    },
    Case {
        component: "hover_card",
        state: "part-flag-info",
        make: || rsx! { PartsCard { parts: vec![flag(FlagTone::Info)] } },
        wait: INTENT,
    },
    Case {
        component: "hover_card",
        state: "part-messages",
        make: || rsx! { PartsCard { parts: vec![messages()] } },
        wait: INTENT,
    },
    Case {
        component: "hover_card",
        state: "part-foot",
        make: || rsx! { PartsCard { parts: vec![foot()] } },
        wait: INTENT,
    },
    Case {
        component: "hover_card",
        state: "part-actions",
        make: || rsx! { PartsCard { parts: vec![actions()] } },
        wait: INTENT,
    },
    Case {
        component: "hover_card",
        state: "sender-parts",
        make: || rsx! { PartsCard { parts: vec![person(), stats(), flag(FlagTone::Danger), foot(), actions()] } },
        wait: INTENT,
    },
    // Tooltip: the fly label, and the card closed and open.
    Case {
        component: "tooltip",
        state: "fly",
        make: || rsx! { Tooltip { kind: TooltipKind::Fly, text: "Snooze until…", ds::IconButton { variant: ds::IconButtonVariant::Strip, icon: Icon::Clock, label: "Snooze", onclick: |_| {} } } },
        wait: NOW,
    },
    Case {
        component: "tooltip",
        state: "card-closed",
        make: || rsx! { Tooltip { kind: TooltipKind::Card, text: "Wed 23 Sep 2026, 09:41", sub: "10:41 their time (Lagos)", "09:41" } },
        wait: NOW,
    },
    Case {
        component: "tooltip",
        state: "card-open",
        make: || rsx! { TipOpen {} },
        wait: INTENT,
    },
    // Toast: hidden (every root), up with an undo, up without one.
    Case {
        component: "toast",
        state: "hidden",
        make: || rsx! {},
        wait: NOW,
    },
    Case {
        component: "toast",
        state: "shown",
        make: || rsx! { Pushed { undo: UndoToken(7) } },
        wait: FRAME,
    },
    Case {
        component: "toast",
        state: "shown-no-undo",
        make: || rsx! { Pushed { undo: None } },
        wait: FRAME,
    },
    // Scrim, Peek (both modes), Sheet.
    Case {
        component: "scrim",
        state: "default",
        make: || rsx! { Scrim { label: "Close peek", onclose: |_| {} } },
        wait: NOW,
    },
    Case {
        component: "peek",
        state: "center",
        make: || rsx! { Peek { mode: PeekMode::Center, label: "Re: UIDL stability", onclose: |_| {}, p { "The reader." } } },
        wait: NOW,
    },
    Case {
        component: "peek",
        state: "full-present",
        make: || rsx! { Peek { mode: PeekMode::Full, label: "Re: UIDL stability", onclose: |_| {}, p { "The reader." } } },
        wait: SETTLED,
    },
    Case {
        component: "sheet",
        state: "default",
        make: || rsx! { Sheet { label: "Accounts", onclose: |_| {}, p { "Settings." } } },
        wait: NOW,
    },
    // CommandPalette: results with a query and tokens, and nothing found.
    Case {
        component: "command_palette",
        state: "results",
        make: || rsx! { CommandPalette { label: "Search and commands", placeholder: "Search mail, people, actions", query: "uidl", tokens: vec!["unread".to_string()], groups: palette_groups(), empty: "Nothing in this Space matches.", oninput: |_| {}, onpick: |_: u8| {}, onclose: |_| {} } },
        wait: NOW,
    },
    Case {
        component: "command_palette",
        state: "empty",
        make: || rsx! { CommandPalette::<u8> { label: "Search and commands", placeholder: "Search mail, people, actions", query: "zz", tokens: Vec::new(), groups: Vec::new(), empty: "Nothing in this Space matches. Search checks subjects, names, addresses and the text of every message.", oninput: |_| {}, onpick: |_| {}, onclose: |_| {} } },
        wait: NOW,
    },
    // LinkPill: honest and lying.
    Case {
        component: "link_pill",
        state: "honest",
        make: || rsx! { LinkPill { target: LinkTarget::Honest { scheme_sub: "https://www.".to_string(), registered: "rfc-editor.org".to_string(), path: "/rfc/rfc1939".to_string() } } },
        wait: NOW,
    },
    Case {
        component: "link_pill",
        state: "lying",
        make: || rsx! { LinkPill { target: LinkTarget::Lying { registered: "g00gle-security.xyz".to_string(), shown: "google.com".to_string() } } },
        wait: NOW,
    },
    // SelectionBubble: formatting actions, and the link field.
    Case {
        component: "selection_bubble",
        state: "actions",
        make: || rsx! { SelectionBubble { anchor: selection(), mode: BubbleMode::Actions(bubble_actions()), onlink: |_| {}, onclose: |_| {} } },
        wait: NOW,
    },
    Case {
        component: "selection_bubble",
        state: "link",
        make: || rsx! { SelectionBubble { anchor: selection(), mode: BubbleMode::Link, onlink: |_| {}, onclose: |_| {} } },
        wait: NOW,
    },
    // SendPill: mounted below the edge, up a frame later, done.
    Case {
        component: "send_pill",
        state: "mounted",
        make: || rsx! { SendPill { text: "Sending in 3 s", progress: Fraction(400), phase: SendPhase::Counting, onundo: |_| {} } },
        wait: NOW,
    },
    Case {
        component: "send_pill",
        state: "counting",
        make: || rsx! { SendPill { text: "Sending in 3 s", progress: Fraction(400), phase: SendPhase::Counting, onundo: |_| {} } },
        wait: FRAME,
    },
    Case {
        component: "send_pill",
        state: "done",
        make: || rsx! { SendPill { text: "Sent", progress: Fraction(1000), phase: SendPhase::Done, onundo: |_| {} } },
        wait: FRAME,
    },
];

/// A Card tooltip whose target the pointer came over.
#[component]
fn TipOpen() -> Element {
    let hub = use_hover_hub();
    use_hook(move || {
        hub.feed(HoverEvent::Over((
            HoverKey("tip:Wed 23 Sep 2026, 09:41".to_string()),
            HoverKind::Sender,
        )))
    });
    rsx! {
        Tooltip { kind: TooltipKind::Card, text: "Wed 23 Sep 2026, 09:41", sub: "10:41 their time (Lagos)", "09:41" }
    }
}
