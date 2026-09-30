//! Overlays: every menu kind, the palette, popovers under each dismiss policy, peek and sheet.
//! The pills, the toast and the hover cards are in `pills.rs`.

use super::pills::{Bubble, Cards, Pills};
use super::{Section, Specimen};
use crate::axes::{Axes, Showcase};
use dioxus::prelude::*;
use ds::{
    Anchor, Arrow, Availability, AvatarFace, AvatarShape, AvatarSize, AvatarTone, Button, Check,
    CommandPalette, Dismiss, Icon, Menu, MenuEntrance, MenuEntry, MenuFilter, MenuKind, MenuTile,
    MenuTrail, MountedRef, Peek, PeekMode, PersonHue, Placement, Point, Popover, Px, Sheet,
    Shortcut, ShortcutKey, Side, use_toast_hub,
};
use ds::{Answers, ControlSize};

/// Everything the page can open, one at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Opened {
    Menu(MenuKind),
    Nested,
    Status,
    Palette,
    Popover(Dismiss, Arrow),
    Peek(PeekMode),
    Sheet,
}

const MENUS: [(MenuKind, &str); 4] = [
    (MenuKind::Rich, "Rich menu"),
    (MenuKind::Slim, "Slim menu"),
    (MenuKind::Dropdown, "Dropdown (type to filter)"),
    (MenuKind::Context, "Context menu"),
];

const POPOVERS: [(Dismiss, Arrow, &str); 3] = [
    (
        Dismiss::Transient,
        Arrow::Arrow,
        "Popover: transient, arrow",
    ),
    (
        Dismiss::Semitransient,
        Arrow::None,
        "Popover: semitransient",
    ),
    (Dismiss::Manual, Arrow::None, "Popover: manual"),
];

/// Where a posed snapshot opens its menu: over the page, clear of the toolbar.
const POSED_AT: Point = Point {
    x: Px(40.0),
    y: Px(330.0),
};

/// Where a posed snapshot opens the submenu specimen: right of the Rich menu.
const POSED_NESTED_AT: Point = Point {
    x: Px(400.0),
    y: Px(330.0),
};

/// Where a posed snapshot opens the bar status menu: the Dropdown hangs left of this point,
/// clear of the submenu specimen.
const POSED_STATUS_AT: Point = Point {
    x: Px(1230.0),
    y: Px(330.0),
};

/// The nested specimen's submenu parent, opened in the posed snapshot (its choice number:
/// items and parents counted, headers and rules not).
const NESTED_OPEN: usize = 2;

/// The overlays page.
#[component]
pub fn OverlaysPage() -> Element {
    let showcase = use_context::<Signal<Axes>>().peek().showcase;
    let mut opened = use_signal(|| match showcase {
        Showcase::Posed => Some(Opened::Menu(MenuKind::Rich)),
        Showcase::Live => None,
    });
    let mut anchor = use_signal(|| None::<MountedRef>);
    let toasts = use_toast_hub();
    use_hook(move || {
        if showcase == Showcase::Posed {
            toasts.push(
                "Archived “Invoice #2291”".to_string(),
                Some(ds::UndoToken(1)),
            );
        }
    });
    let at = match (showcase, anchor()) {
        (Showcase::Live, Some(mounted)) => Anchor::Mounted(mounted),
        _ => Anchor::Point(POSED_AT),
    };
    let close = move |_| opened.set(None);
    let button = move |what: Opened, label: &'static str| {
        rsx! {
            Button {
                label,
                value: Some(if opened() == Some(what) { Check::On } else { Check::Off }),
                onclick: move |_| opened.set(Some(what)),
            }
        }
    };
    rsx! {
        Section { title: "Menus", note: "One menu, four kinds. Arrow keys wrap and skip disabled items, Enter or Tab picks, Escape or an outside click closes. A submenu opens on a 200 ms rest, or at once on Right, Enter or a click; Left or Escape closes it.",
            div {
                class: "g-row",
                onmounted: move |event| anchor.set(Some(MountedRef(event.data()))),
                for (kind , label) in MENUS {
                    {button(Opened::Menu(kind), label)}
                }
                {button(Opened::Nested, "Submenus and disabled items")}
                {button(Opened::Status, "Bar status menu (status lines, no entrance)")}
            }
        }
        Section { title: "Palette, popovers, peek, sheet",
            div { class: "g-row",
                {button(Opened::Palette, "Command palette")}
                for (dismiss , arrow , label) in POPOVERS {
                    {button(Opened::Popover(dismiss, arrow), label)}
                }
                {button(Opened::Peek(PeekMode::Center), "Peek: center")}
                {button(Opened::Peek(PeekMode::Full), "Peek: full")}
                {button(Opened::Sheet, "Sheet")}
            }
        }
        Cards {}
        Bubble {}
        Pills { showcase }
        super::launcher::EmbeddedPalette {}
        super::launcher_hints::SpotlightHints {}
        super::overlays_mailo::RecentPalette {}
        super::overlays_mailo::FieldMenu {}
        super::overlays_mailo4::HookKeyedCards {}
        super::overlays_mailo4::LabelChecklist {}
        super::overlays_mailo4::InlineActions {}
        super::overlays_mailo5::FieldFilterMenu {}
        super::control_center::ControlCenter {}
        super::overlays_sheet::PowerMenu {}
        super::overlays_alert::Alerts {}
        super::overlays_notifications::Notifications {}
        super::calendar::Calendar {}
        super::widgets::Widgets {}
        super::overlays_shot::ShotThumbnails {}
        if showcase == Showcase::Posed {
            Menu::<u8> {
                kind: MenuKind::Context,
                anchor: Anchor::Point(POSED_NESTED_AT),
                entries: nested(),
                expanded: Some(NESTED_OPEN),
                onpick: move |_| {},
                onclose: move |_| {},
            }
            Menu::<u8> {
                kind: MenuKind::Dropdown,
                anchor: Anchor::Point(POSED_STATUS_AT),
                entries: status(),
                entrance: MenuEntrance::Instant,
                onpick: move |_| {},
                onclose: move |_| {},
            }
        }
        match opened() {
            Some(Opened::Menu(kind)) => rsx! {
                Menu::<u8> {
                    key: "{kind:?}",
                    kind,
                    anchor: at.clone(),
                    entries: entries(kind),
                    filter: if kind == MenuKind::Dropdown { MenuFilter::Typing } else { MenuFilter::None },
                    onpick: move |_| {},
                    onclose: close,
                }
            },
            Some(Opened::Nested) => rsx! {
                Menu::<u8> {
                    kind: MenuKind::Context,
                    anchor: at.clone(),
                    entries: nested(),
                    onpick: move |_| {},
                    onclose: close,
                }
            },
            Some(Opened::Status) => rsx! {
                Menu::<u8> {
                    kind: MenuKind::Dropdown,
                    anchor: at.clone(),
                    entries: status(),
                    entrance: MenuEntrance::Instant,
                    onpick: move |_| {},
                    onclose: close,
                }
            },
            Some(Opened::Palette) => rsx! { Palette { onclose: close } },
            Some(Opened::Popover(dismiss, arrow)) => rsx! {
                Popover {
                    key: "{dismiss:?}",
                    anchor: at.clone(),
                    placement: Placement::new(Side::Bottom, ds::Align::Start),
                    gap: Px(8.0),
                    arrow,
                    dismiss,
                    onclose: close,
                    div { class: "g-panel",
                        Specimen { name: format!("{dismiss:?} dismiss"), code: "Transient: Escape or an outside click closes; Semitransient: Escape only; Manual: its owner".to_string(),
                            Button { size: ControlSize::Mini, label: "Close", onclick: move |_| opened.set(None) }
                        }
                    }
                }
            },
            Some(Opened::Peek(mode)) => rsx! {
                Peek { key: "{mode:?}", mode, label: "Thread peek", onclose: close,
                    div { class: "g-panel",
                        h2 { "Re: UIDL stability across servers" }
                        p { class: "g-note", "The reader in a panel over the card. The scrim, the close tool or Escape closes it." }
                    }
                }
            },
            Some(Opened::Sheet) => rsx! {
                Sheet { label: "Settings", onclose: close,
                    div { class: "g-panel",
                        h2 { "A sheet" }
                        p { class: "g-note", "Hangs from the top edge and dims nothing; Escape closes it." }
                        Button { answers: Answers::Return, label: "Done", onclick: move |_| opened.set(None) }
                    }
                }
            },
            None => rsx! {},
        }
    }
}

/// A bar status menu (bar gaps): status lines that are never choices, a rule, then the items.
fn status() -> Vec<MenuEntry<u8>> {
    let item = |value: u8, title: &str| MenuEntry::Item {
        availability: Availability::Enabled,
        value,
        title: title.to_string(),
        detail: None,
        tile: None,
        trail: MenuTrail::None,
        check: None,
    };
    vec![
        MenuEntry::Info {
            title: "Wired: connected".to_string(),
            detail: Some("192.168.1.4 · 1 Gb/s".to_string()),
        },
        MenuEntry::Info {
            title: "VPN: off".to_string(),
            detail: None,
        },
        MenuEntry::Separator,
        item(1, "Disconnect"),
        item(2, "Network settings…"),
    ]
}

/// A person's face for a menu tile.
fn face(name: &str) -> AvatarFace {
    AvatarFace {
        initial: name.chars().next().unwrap_or('?'),
        size: AvatarSize::Size20,
        tone: AvatarTone::Person(PersonHue::of(name)),
        shape: AvatarShape::Round,
    }
}

/// What each kind lists: tiles for the rich and slim kinds, checks for the dropdown.
fn entries(kind: MenuKind) -> Vec<MenuEntry<u8>> {
    let item =
        |value: u8, title: &str, detail: Option<&str>, tile: Option<MenuTile>, trail: MenuTrail| {
            MenuEntry::Item {
                availability: Availability::Enabled,
                value,
                title: title.to_string(),
                detail: detail.map(str::to_string),
                tile,
                trail,
                check: match kind {
                    MenuKind::Dropdown => Some(if value == 1 { Check::On } else { Check::Off }),
                    MenuKind::Rich | MenuKind::Slim | MenuKind::Context => None,
                },
            }
        };
    let shortcut = |c| MenuTrail::Shortcut(Shortcut(vec![ShortcutKey::Ctrl, ShortcutKey::Char(c)]));
    vec![
        MenuEntry::Header("Snooze".to_string()),
        item(
            0,
            "Later today",
            Some("18:00"),
            Some(MenuTile::Icon(Icon::Clock)),
            shortcut('l'),
        ),
        item(
            1,
            "Tomorrow",
            Some("Friday 08:00"),
            Some(MenuTile::Icon(Icon::Sun)),
            shortcut('t'),
        ),
        item(
            2,
            "Next week",
            None,
            Some(MenuTile::Text("Mo".to_string())),
            MenuTrail::Note("Mon".to_string()),
        ),
        MenuEntry::Separator,
        MenuEntry::Header("Send to".to_string()),
        item(
            3,
            "Dana Okafor",
            Some("dana@example.org"),
            Some(MenuTile::Avatar(face("Dana Okafor"))),
            MenuTrail::None,
        ),
        item(
            4,
            "Priya Raman",
            Some("priya@example.org"),
            Some(MenuTile::Avatar(face("Priya Raman"))),
            MenuTrail::None,
        ),
    ]
}

/// A tray menu's shape: a disabled item, a submenu with a disabled child, a disabled submenu.
fn nested() -> Vec<MenuEntry<u8>> {
    let item = |value: u8, title: &str, icon: Option<Icon>, availability| MenuEntry::Item {
        value,
        title: title.to_string(),
        detail: None,
        tile: icon.map(MenuTile::Icon),
        trail: MenuTrail::None,
        check: None,
        availability,
    };
    vec![
        MenuEntry::Header("Toshy".to_string()),
        item(
            0,
            "Open preferences",
            Some(Icon::Settings),
            Availability::Enabled,
        ),
        item(
            1,
            "Pause remapping",
            Some(Icon::Clock),
            Availability::Disabled,
        ),
        MenuEntry::Separator,
        MenuEntry::Submenu {
            title: "Keyboard type".to_string(),
            tile: Some(MenuTile::Icon(Icon::Command)),
            availability: Availability::Enabled,
            children: vec![
                item(10, "Automatic", None, Availability::Enabled),
                item(11, "Apple", None, Availability::Enabled),
                item(12, "Chromebook", None, Availability::Disabled),
                item(13, "Windows", None, Availability::Enabled),
            ],
        },
        MenuEntry::Submenu {
            title: "Services".to_string(),
            tile: Some(MenuTile::Icon(Icon::Refresh)),
            availability: Availability::Disabled,
            children: vec![item(20, "Restart", None, Availability::Enabled)],
        },
        item(2, "Quit", Some(Icon::X), Availability::Enabled),
    ]
}

/// The palette, holding its own query.
#[component]
fn Palette(onclose: EventHandler<()>) -> Element {
    let mut query = use_signal(String::new);
    let item = |value: u8, title: &str, icon: Icon, keys: Vec<ShortcutKey>| MenuEntry::Item {
        availability: Availability::Enabled,
        value,
        title: title.to_string(),
        detail: None,
        tile: Some(MenuTile::Icon(icon)),
        trail: MenuTrail::Shortcut(Shortcut(keys)),
        check: None,
    };
    let actions = vec![
        item(
            0,
            "Compose a message",
            Icon::Pen,
            vec![ShortcutKey::Char('c')],
        ),
        item(1, "Archive", Icon::Archive, vec![ShortcutKey::Char('e')]),
        item(
            2,
            "Snooze until tomorrow",
            Icon::Clock,
            vec![ShortcutKey::Char('h')],
        ),
    ];
    let places = vec![
        item(
            3,
            "Go to Inbox",
            Icon::Inbox,
            vec![ShortcutKey::Char('g'), ShortcutKey::Char('i')],
        ),
        item(
            4,
            "Go to Starred",
            Icon::Star,
            vec![ShortcutKey::Char('g'), ShortcutKey::Char('s')],
        ),
    ];
    let typed = query();
    let keep = |entries: Vec<MenuEntry<u8>>| -> Vec<MenuEntry<u8>> {
        entries
            .into_iter()
            .filter(|entry| match entry {
                MenuEntry::Item { title, .. } | MenuEntry::Submenu { title, .. } => {
                    title.to_lowercase().contains(&typed.to_lowercase())
                }
                MenuEntry::Row(row) => row
                    .title
                    .plain_text()
                    .to_lowercase()
                    .contains(&typed.to_lowercase()),
                MenuEntry::Header(_) | MenuEntry::Info { .. } | MenuEntry::Separator => true,
            })
            .collect()
    };
    rsx! {
        CommandPalette::<u8> {
            label: "Command palette",
            placeholder: "Type a command",
            query: typed.clone(),
            tokens: Vec::new(),
            groups: vec![("Actions".to_string(), keep(actions)), ("Places".to_string(), keep(places))],
            empty: "Nothing matches.",
            oninput: move |next| query.set(next),
            onpick: move |_| {},
            onclose,
        }
    }
}
