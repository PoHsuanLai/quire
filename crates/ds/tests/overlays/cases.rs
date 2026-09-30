//! Every overlay component in every state, as data: the table the golden test walks. Each case
//! renders inside a `Ds`, so floating surfaces arrive through its overlay host; `wait` lets the
//! case's timers run first (a hover card's 450 ms intent, an entrance settling).

use dioxus::prelude::*;
use ds::ControlSize;
use ds::components::overlays::sheet_width::SheetWidth;
use ds::{Accessory, Align, Availability, Button, RowLeading};
use ds::{
    Anchor, Arrow, AvatarFace, AvatarShape, AvatarSize, AvatarTone, CommandPalette, CommandPaletteHost, Dismiss, ExternalIcon, FlagTone, Glyph,
    HoverCard, HoverCardPart, HoverEvent, HoverKey, HoverKind, HoverMessage, HoverProfile,
    HoverStat, HoverTarget, Icon, IconSize, IconSource, IconUrl, LinkPill, LinkTarget,
    Menu, MenuImage, MenuItem, MenuPlacement, PaletteGroup, PaletteRow, Peek, PeekMode, PersonHue,
    Placement, Point, Popover, Px, Rect, SendPhase, SendPill, Sheet, Shown, Side,
    Size, Tooltip, UndoToken, use_hover_hub, use_toasts,
};

use ds::{
    Answers, Attach, Check, EmptyForm, EmptyState, Fraction, Shortcut, ShortcutKey, SidePanel,
    Skeleton, SkeletonShape,
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

fn item(value: u8, title: &str) -> MenuItem<u8> {
    MenuItem::new(value, title)
}

/// A bar status menu (bar gaps): two status lines, a rule, one item.
fn status_lines() -> Vec<MenuItem<u8>> {
    vec![
        MenuItem::Info {
            title: "Wired: connected".to_string(),
            detail: Some("192.168.1.4".to_string()),
        },
        MenuItem::Info {
            title: "Battery 82%".to_string(),
            detail: None,
        },
        MenuItem::Separator,
        item(1, "Network settings…"),
    ]
}

/// The file menu: a header, items with images, key equivalents and every check state, a rule,
/// one more.
fn snooze() -> Vec<MenuItem<u8>> {
    vec![
        MenuItem::Header("Snooze until".to_string()),
        item(1, "Later today")
            .with_image(MenuImage::Icon(Icon::Clock))
            .with_key(Shortcut(vec![ShortcutKey::Ctrl, ShortcutKey::Char('l')])),
        item(2, "Tomorrow").with_check(Check::On),
        item(3, "Next week").with_check(Check::Off),
        item(4, "Any day").with_check(Check::Mixed),
        MenuItem::Separator,
        item(5, "Pick a date…"),
    ]
}

fn group_by() -> Vec<MenuItem<u8>> {
    vec![
        item(1, "Date").with_check(Check::On),
        item(2, "Sender").with_check(Check::Off),
    ]
}

/// A disabled item, a submenu parent and a disabled submenu parent.
fn nested() -> Vec<MenuItem<u8>> {
    vec![
        item(1, "Open"),
        item(2, "Pause").with_availability(Availability::Disabled),
        MenuItem::Submenu {
            title: "More".to_string(),
            image: Some(MenuImage::Icon(Icon::Settings)),
            availability: Availability::Enabled,
            children: vec![item(10, "About")],
        },
        MenuItem::Submenu {
            title: "Services".to_string(),
            image: None,
            availability: Availability::Disabled,
            children: vec![item(20, "Restart")],
        },
    ]
}

/// Applications with their own icons: an image and a symbolic one.
fn app_groups() -> Vec<PaletteGroup<u8>> {
    let app = |value: u8, title: &str, icon: IconSource| PaletteRow {
        leading: RowLeading::Source(icon),
        ..PaletteRow::new(value, title)
    };
    vec![PaletteGroup::list(
        "Applications",
        vec![
            app(
                1,
                "Firefox",
                IconSource::Image(ExternalIcon {
                    url: IconUrl::png(b"\x89PNG"),
                    size: IconSize::Tile48,
                }),
            ),
            app(
                2,
                "Files",
                IconSource::Symbolic(ExternalIcon {
                    url: IconUrl::svg("<svg xmlns='http://www.w3.org/2000/svg'/>"),
                    size: IconSize::Tile,
                }),
            ),
        ],
    )]
}

fn palette_groups() -> Vec<PaletteGroup<u8>> {
    vec![
        PaletteGroup::list(
            "Top hit",
            vec![PaletteRow {
                detail: Some("Treat UIDL as a hint, not an identity".into()),
                leading: RowLeading::Avatar(AvatarFace {
                    size: AvatarSize::Size34,
                    ..DANA
                }),
                accessory: Accessory::Text(
                    Shortcut(vec![ShortcutKey::Ctrl, ShortcutKey::Char('1')]).glyphs(),
                ),
                ..PaletteRow::new(1, "Re: UIDL stability")
            }],
        ),
        PaletteGroup::list("People", Vec::new()),
        PaletteGroup::list(
            "Actions",
            vec![PaletteRow {
                leading: RowLeading::Icon(Icon::Refresh),
                ..PaletteRow::new(2, "Sync now")
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
        move || {
            hub.feed(HoverEvent::Over(
                (key, HoverProfile::Card),
                HoverProfile::Card,
            ))
        }
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
                    span { class: "ds-hovercard-keys", ds::KeyEquivalent { shortcut: Shortcut(vec![ShortcutKey::Space]), style: ds::KeyStyle::Cap } " peek" }
                }
                div { class: "ds-hovercard-actions",
                    Button { size: ControlSize::Mini, label: "Reply", onclick: |_| {} }
                }
            }
        }
    }
}

/// A sender card composed from parts, open: the same card [`SenderCard`] writes by hand.
#[component]
pub fn PartsCard(parts: Vec<HoverCardPart>) -> Element {
    let hub = use_hover_hub();
    let key = HoverKey("sender:3".to_string());
    use_hook({
        let key = key.clone();
        move || {
            hub.feed(HoverEvent::Over(
                (key, HoverProfile::Card),
                HoverProfile::Card,
            ))
        }
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
        key: Some((Shortcut(vec![ShortcutKey::Space]), "peek".to_string())),
    }
}

fn actions() -> HoverCardPart {
    HoverCardPart::Actions(vec![rsx! {
        Button { size: ControlSize::Mini, label: "Reply", onclick: |_| {} }
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

pub const CASES: &[Case] = &[
    // Menu: each placement, an empty list, disabled items and submenus, status lines.
    Case {
        component: "menu",
        state: "popup",
        make: || rsx! { Menu { placement: MenuPlacement::Popup, anchor: Anchor::Rect(button_rect()), items: snooze(), onpick: |_| {}, onclose: |_| {} } },
        wait: NOW,
    },
    Case {
        component: "menu",
        state: "popup-present",
        make: || rsx! { Menu { placement: MenuPlacement::Popup, anchor: Anchor::Rect(button_rect()), items: snooze(), onpick: |_| {}, onclose: |_| {} } },
        wait: SETTLED,
    },
    Case {
        component: "menu",
        state: "bar",
        make: || rsx! { Menu { placement: MenuPlacement::Bar, anchor: Anchor::Rect(button_rect()), items: group_by(), onpick: |_| {}, onclose: |_| {} } },
        wait: NOW,
    },
    Case {
        component: "menu",
        state: "context",
        make: || rsx! { Menu { placement: MenuPlacement::Context, anchor: Anchor::Point(Point { x: Px(420.0), y: Px(310.0) }), items: vec![item(1, "Archive"), item(2, "Snooze…")], onpick: |_| {}, onclose: |_| {} } },
        wait: NOW,
    },
    Case {
        component: "menu",
        state: "context-hides-the-unavailable",
        make: || rsx! { Menu { placement: MenuPlacement::Context, anchor: Anchor::Point(Point { x: Px(40.0), y: Px(40.0) }), items: nested(), onpick: |_| {}, onclose: |_| {} } },
        wait: NOW,
    },
    Case {
        component: "menu",
        state: "empty",
        make: || rsx! { Menu::<u8> { placement: MenuPlacement::Popup, anchor: Anchor::Rect(button_rect()), items: Vec::new(), onpick: |_| {}, onclose: |_| {} } },
        wait: NOW,
    },
    Case {
        component: "menu",
        state: "disabled-and-submenu",
        make: || rsx! { Menu { placement: MenuPlacement::Popup, anchor: Anchor::Rect(button_rect()), items: nested(), onpick: |_| {}, onclose: |_| {} } },
        wait: NOW,
    },
    Case {
        component: "menu",
        state: "bar-status-lines",
        make: || rsx! { Menu { placement: MenuPlacement::Bar, anchor: Anchor::Rect(button_rect()), items: status_lines(), onpick: |_| {}, onclose: |_| {} } },
        wait: NOW,
    },
    // Popover: each dismiss policy, and the arrow.
    Case {
        component: "popover",
        state: "transient",
        make: || rsx! { Popover { anchor: Anchor::Rect(button_rect()), placement: Placement::new(Side::Bottom, Align::Start), gap: Px(6.0), onclose: |_| {}, "Anything" } },
        wait: NOW,
    },
    Case {
        component: "popover",
        state: "semitransient-esc-only",
        make: || rsx! { Popover { anchor: Anchor::Rect(button_rect()), placement: Placement::new(Side::Top, Align::Center), gap: Px(8.0), dismiss: Dismiss::Semitransient, onclose: |_| {}, "Anything" } },
        wait: NOW,
    },
    Case {
        component: "popover",
        state: "manual-owner-closes",
        make: || rsx! { Popover { anchor: Anchor::Point(Point { x: Px(40.0), y: Px(40.0) }), placement: Placement::new(Side::Right, Align::End), gap: Px(0.0), dismiss: Dismiss::Manual, onclose: |_| {}, "Anything" } },
        wait: NOW,
    },
    Case {
        component: "popover",
        state: "arrow",
        make: || rsx! { Popover { anchor: Anchor::Rect(button_rect()), placement: Placement::new(Side::Bottom, Align::Center), gap: Px(2.0), arrow: Arrow::Arrow, onclose: |_| {}, "Anything" } },
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
    // Tooltip: at rest under the pointer's wait, and caller-driven up and down.
    Case {
        component: "tooltip",
        state: "rest",
        make: || rsx! { Tooltip { text: "Snooze until…", ds::Button { bezel: ds::Bezel::Toolbar, image: ds::ImagePosition::Only, icon: Icon::Clock, label: "Snooze", onclick: |_| {} } } },
        wait: NOW,
    },
    // Caller-driven: shown with no pointer, hidden under one.
    Case {
        component: "tooltip",
        state: "shown",
        make: || rsx! { Tooltip { text: "Terminal", shown: Some(Shown::Visible), span { "tile" } } },
        wait: NOW,
    },
    Case {
        component: "tooltip",
        state: "hidden",
        make: || rsx! { Tooltip { text: "Terminal", shown: Some(Shown::Hidden), span { "tile" } } },
        wait: NOW,
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
    // Peek (both modes), Sheet.
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
        state: "window",
        make: || rsx! { Sheet { label: "Accounts", onclose: |_| {}, p { "Settings." } } },
        wait: NOW,
    },
    // Sheet and modal parts: centred, shown by its host, and mounted hidden.
    Case {
        component: "sheet",
        state: "centre",
        make: || rsx! { Sheet { label: "Power", onclose: |_| {}, attach: Attach::Centre, p { "Shut down?" } } },
        wait: SETTLED,
    },
    Case {
        component: "sheet",
        state: "shown",
        make: || rsx! { Sheet { label: "Power", onclose: |_| {}, shown: Shown::Visible, on_hidden: |_| {}, p { "Shut down?" } } },
        wait: NOW,
    },
    Case {
        component: "sheet",
        state: "mounted-hidden",
        make: || rsx! { Sheet { label: "Power", onclose: |_| {}, shown: Shown::Hidden, on_hidden: |_| {}, p { "Shut down?" } } },
        wait: NOW,
    },
    Case {
        component: "sheet",
        state: "bottom-wide",
        make: || rsx! { Sheet { label: "Edit Widgets", onclose: |_| {}, attach: Attach::Bottom, width: SheetWidth::Wide, p { "Gallery." } } },
        wait: SETTLED,
    },
    Case {
        component: "sheet",
        state: "narrow",
        make: || rsx! { Sheet { label: "Alert", onclose: |_| {}, attach: Attach::Centre, width: SheetWidth::Narrow, p { "Password?" } } },
        wait: SETTLED,
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
        make: || rsx! { CommandPalette::<u8> { label: "Search and commands", placeholder: "Search mail, people, actions", query: "zz", tokens: Vec::new(), groups: ds::PaletteGroups::default(), empty: "Nothing in this Space matches. Search checks subjects, names, addresses and the text of every message.", oninput: |_| {}, onpick: |_| {}, onclose: |_| {} } },
        wait: NOW,
    },
    // Embedded in a launcher surface: no scrim, `cmdk-in`, the card's id;
    // and a row whose tile is an app's icon.
    Case {
        component: "command_palette",
        state: "surface-cmdk",
        make: || rsx! { CommandPalette { label: "Launch", placeholder: "Search apps, windows, actions", query: "", tokens: Vec::new(), groups: palette_groups(), empty: "Nothing matches.", oninput: |_| {}, onpick: |_: u8| {}, onclose: |_| {}, host: CommandPaletteHost::Surface, id: "launcher-card" } },
        wait: NOW,
    },
    Case {
        component: "command_palette",
        state: "app-icon",
        make: || rsx! { CommandPalette { label: "Launch", placeholder: "Search apps, windows, actions", query: "fi", tokens: Vec::new(), groups: app_groups(), empty: "Nothing matches.", oninput: |_| {}, onpick: |_: u8| {}, onclose: |_| {}, host: CommandPaletteHost::Surface } },
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
    // EmptyState: each form, with an action and with Retry.
    Case {
        component: "empty_state",
        state: "empty-action",
        make: || rsx! { EmptyState { title: "No messages", description: Some("Mail you receive lands here.".to_string()), action: Some(rsx! { Button { answers: Answers::Return, label: "Compose", onclick: |_| {} } }) } },
        wait: NOW,
    },
    Case {
        component: "empty_state",
        state: "no-results",
        make: || rsx! { EmptyState { form: EmptyForm::NoResults, title: "No results for “uidl”" } },
        wait: NOW,
    },
    Case {
        component: "empty_state",
        state: "failure-retry",
        make: || rsx! { EmptyState { form: EmptyForm::Failure, title: "Couldn’t load your mail", description: Some("The server didn’t answer.".to_string()), onretry: |_| {} } },
        wait: NOW,
    },
    Case {
        component: "empty_state",
        state: "retry-only-on-failure",
        make: || rsx! { EmptyState { title: "No messages", onretry: |_| {} } },
        wait: NOW,
    },
    // Skeleton: each shape, sized, and hidden.
    Case {
        component: "skeleton",
        state: "line",
        make: || rsx! { Skeleton {} },
        wait: NOW,
    },
    Case {
        component: "skeleton",
        state: "line-sized",
        make: || rsx! { Skeleton { width: Some(Px(120.0)) } },
        wait: NOW,
    },
    Case {
        component: "skeleton",
        state: "block",
        make: || rsx! { Skeleton { shape: SkeletonShape::Block, width: Some(Px(160.0)), height: Some(Px(80.0)) } },
        wait: NOW,
    },
    Case {
        component: "skeleton",
        state: "circle",
        make: || rsx! { Skeleton { shape: SkeletonShape::Circle, width: Some(Px(40.0)), height: Some(Px(99.0)) } },
        wait: NOW,
    },
    Case {
        component: "skeleton",
        state: "hidden",
        make: || rsx! { Skeleton { shown: Shown::Hidden } },
        wait: NOW,
    },
    // SidePanel: shown, with a header, and mounted hidden.
    Case {
        component: "side_panel",
        state: "shown",
        make: || rsx! { SidePanel { label: "Notification Center", shown: Shown::Visible, p { "Nothing new." } } },
        wait: NOW,
    },
    Case {
        component: "side_panel",
        state: "header",
        make: || rsx! { SidePanel { label: "Notification Center", shown: Shown::Visible, header: Some(rsx! { strong { "Today" } }), p { "Nothing new." } } },
        wait: NOW,
    },
    Case {
        component: "side_panel",
        state: "hidden",
        make: || rsx! { SidePanel { label: "Notification Center", shown: Shown::Hidden, p { "Nothing new." } } },
        wait: NOW,
    },
];
