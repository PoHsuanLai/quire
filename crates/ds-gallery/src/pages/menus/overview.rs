//! Menus: `Menu` in its three placements, every `MenuItem` state (drawn inline with the highlight
//! posed by a controlled cursor), and `PopUpButton` (design/30 section 2.4 and 2.1).

use crate::axes::{Axes, Showcase};
use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::menus::item::item::MenuImage;
use ds::components::menus::pop_up_button::{PopUpButton, PopUpKind};
use ds::host::measure::{Anchor, MountedRef};
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;

/// The three placements, one at a time.
const PLACEMENTS: [(MenuPlacement, &str); 3] = [
    (MenuPlacement::Bar, "Bar menu"),
    (MenuPlacement::Popup, "Pop-up menu"),
    (MenuPlacement::Context, "Context menu"),
];

/// The nested specimen's submenu parent, opened in the posed snapshot (its choice number).
const NESTED_OPEN: usize = 2;

/// What a menu can open, one at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Opened {
    Placement(MenuPlacement),
    Nested,
    Status,
}

/// The menus page.
#[component]
pub fn MenusPage() -> Element {
    rsx! {
        Menus {}
        Items {}
        crate::pages::menus::pick::HintsAndPickList {}
        crate::pages::menus::field::FieldAndCard {}
        crate::pages::menus::search::SearchSuggestions {}
        PopUps {}
    }
}

fn keys(key: char) -> Shortcut {
    Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char(key)])
}

/// A file menu's shape: key equivalents, a disabled item, checks, rules.
fn file_menu() -> Vec<MenuItem<u8>> {
    vec![
        MenuItem::Header("File".to_string()),
        MenuItem::new(1, "New Window")
            .with_key(keys('N'))
            .with_image(MenuImage::Icon(Icon::Plus)),
        MenuItem::new(2, "New Tab").with_key(keys('T')),
        MenuItem::new(3, "Open Recent").with_availability(Availability::Disabled),
        MenuItem::Separator,
        MenuItem::new(4, "Show Sidebar").with_check(Check::On),
        MenuItem::new(5, "Show Path Bar").with_check(Check::Off),
        MenuItem::new(6, "Show Hidden Files").with_check(Check::Mixed),
        MenuItem::Separator,
        MenuItem::new(7, "Close Window").with_key(keys('W')),
    ]
}

/// A tray menu's shape: a disabled item, a submenu with a disabled child, a disabled submenu.
fn nested() -> Vec<MenuItem<u8>> {
    let item = |value: u8, title: &str, icon: Option<Icon>, availability| {
        let item = MenuItem::new(value, title).with_availability(availability);
        match icon {
            Some(icon) => item.with_image(MenuImage::Icon(icon)),
            None => item,
        }
    };
    vec![
        MenuItem::Header("Toshy".to_string()),
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
        MenuItem::Separator,
        MenuItem::Submenu {
            title: "Keyboard type".to_string(),
            image: Some(MenuImage::Icon(Icon::Command)),
            availability: Availability::Enabled,
            children: vec![
                item(10, "Automatic", None, Availability::Enabled),
                item(11, "Apple", None, Availability::Enabled),
                item(12, "Chromebook", None, Availability::Disabled),
                item(13, "Windows", None, Availability::Enabled),
            ],
        },
        MenuItem::Submenu {
            title: "Services".to_string(),
            image: Some(MenuImage::Icon(Icon::Refresh)),
            availability: Availability::Disabled,
            children: vec![item(20, "Restart", None, Availability::Enabled)],
        },
        item(2, "Quit", Some(Icon::X), Availability::Enabled),
    ]
}

/// A status menu (bar gaps): status lines that are never choices, a rule, then the items.
fn status() -> Vec<MenuItem<u8>> {
    vec![
        MenuItem::Info {
            title: "Wired: connected".to_string(),
            detail: Some("192.168.1.4 · 1 Gb/s".to_string()),
        },
        MenuItem::Info {
            title: "VPN: off".to_string(),
            detail: None,
        },
        MenuItem::Separator,
        MenuItem::new(1, "Disconnect"),
        MenuItem::new(2, "Network settings…"),
    ]
}

/// The three placements, opened by buttons.
#[component]
fn Menus() -> Element {
    let showcase = use_context::<Signal<Axes>>().peek().showcase;
    let mut opened = use_signal(|| None::<Opened>);
    let mut anchor = use_signal(|| None::<MountedRef>);
    let mut picked = use_signal(|| "nothing yet".to_string());
    let at = anchor().map_or(Anchor::Point(Point::default()), Anchor::Mounted);
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
        Section {
            title: "Menu",
            note: "One menu, three placements. Arrow keys wrap and skip disabled items, Home and End jump to the ends, letters jump to the next item that starts with them, Enter or Tab picks, Escape or an outside click closes. A menu opens at once; a pick blinks its item twice, then the menu fades out over --t-quick. A submenu opens on a 200 ms rest, or at once on Right, Enter or a click; Left or Escape closes it. A context menu leaves out what is unavailable and shows no key equivalents.",
            div {
                class: "g-row",
                onmounted: move |event| anchor.set(Some(MountedRef(event.data()))),
                for (placement , label) in PLACEMENTS {
                    {button(Opened::Placement(placement), label)}
                }
                {button(Opened::Nested, "Submenus and disabled items")}
                {button(Opened::Status, "Status lines")}
            }
            p { class: "g-note", "Picked: {picked}" }
        }
        if showcase == Showcase::Posed {
            // Each posed-open specimen has its own reserved stage, drawn inline so nothing
            // overlays the page text and nothing waits on a measured anchor.
            div { class: "g-stage-row",
                for (caption , items , at , expanded) in [
                    ("Pop-up menu", file_menu(), Some(0), None),
                    ("Submenu open", nested(), Some(NESTED_OPEN), Some(NESTED_OPEN)),
                    ("Status lines", status(), None, None),
                ] {
                    div { class: "g-stage", "data-wide": (expanded.is_some()).then_some("true"),
                        div { class: "g-menu-card",
                            Menu::<u8> {
                                placement: MenuPlacement::Popup,
                                anchor: Anchor::Point(Point::default()),
                                items,
                                flow: Flow::Inline,
                                active: MenuCursor::Controlled(at),
                                expanded,
                                onpick: |_| {},
                                onclose: |_| {},
                            }
                        }
                        p { class: "g-note g-stage-caption", "{caption}" }
                    }
                }
            }
        }
        match opened() {
            Some(Opened::Placement(placement)) => rsx! {
                Menu::<u8> {
                    key: "{placement:?}",
                    placement,
                    anchor: at.clone(),
                    items: file_menu(),
                    onpick: move |value: u8| picked.set(format!("item {value}")),
                    onclose: close,
                }
            },
            Some(Opened::Nested) => rsx! {
                Menu::<u8> {
                    placement: MenuPlacement::Popup,
                    anchor: at.clone(),
                    items: nested(),
                    onpick: move |value: u8| picked.set(format!("item {value}")),
                    onclose: close,
                }
            },
            Some(Opened::Status) => rsx! {
                Menu::<u8> {
                    placement: MenuPlacement::Bar,
                    anchor: at.clone(),
                    items: status(),
                    onpick: move |value: u8| picked.set(format!("item {value}")),
                    onclose: close,
                }
            },
            None => rsx! {},
        }
    }
}

/// One inline menu with its highlight posed on choice `at` (none for no highlight).
#[component]
fn Posed(
    items: Vec<MenuItem<u8>>,
    at: Option<usize>,
    #[props(default)] expanded: Option<usize>,
) -> Element {
    rsx! {
        div { class: "g-menu-card",
            Menu::<u8> {
                placement: MenuPlacement::Popup,
                anchor: Anchor::Point(Point::default()),
                items,
                flow: Flow::Inline,
                active: MenuCursor::Controlled(at),
                expanded,
                onpick: |_| {},
                onclose: |_| {},
            }
        }
    }
}

/// One specimen: its caption, its items and the posed highlight.
type PosedItems = (&'static str, Vec<MenuItem<u8>>, Option<usize>);

/// Every state of an item, drawn in an inline menu whose highlight is posed.
#[component]
fn Items() -> Element {
    let leaf = |value: u8, title: &str| MenuItem::new(value, title);
    let states: Vec<PosedItems> = vec![
        ("Plain", vec![leaf(0, "Rename"), leaf(1, "Duplicate")], None),
        (
            "Highlighted",
            vec![leaf(0, "Rename"), leaf(1, "Duplicate")],
            Some(1),
        ),
        (
            "Disabled",
            vec![
                leaf(0, "Rename").with_availability(Availability::Disabled),
                leaf(1, "Duplicate"),
            ],
            None,
        ),
        (
            "Checks: on, off, mixed",
            vec![
                leaf(0, "Bold").with_check(Check::On),
                leaf(1, "Italic").with_check(Check::Off),
                leaf(2, "Underline").with_check(Check::Mixed),
            ],
            Some(2),
        ),
        (
            "Image and key equivalent",
            vec![
                leaf(0, "New Window")
                    .with_image(MenuImage::Icon(Icon::Plus))
                    .with_key(keys('N')),
                leaf(1, "Close").with_key(keys('W')),
            ],
            Some(0),
        ),
        (
            "Submenu parent",
            vec![
                leaf(0, "Open"),
                MenuItem::Submenu {
                    title: "Open With".to_string(),
                    image: None,
                    availability: Availability::Enabled,
                    children: vec![leaf(10, "Preview")],
                },
            ],
            Some(1),
        ),
        (
            "Header, status line, rule",
            vec![
                MenuItem::Header("Network".to_string()),
                MenuItem::Info {
                    title: "Wired: connected".to_string(),
                    detail: Some("1 Gb/s".to_string()),
                },
                MenuItem::Separator,
                leaf(0, "Disconnect"),
            ],
            None,
        ),
    ];
    rsx! {
        Section {
            title: "MenuItem",
            note: "The highlight is a fill that snaps, with no fade: the accent under its ink while the window is active, grey when it is not. A menu tracks the pointer, so an item has no hover of its own. The state column keeps a column for the check (a dash for mixed) wherever some item has one; the image column likewise.",
            div { class: "g-row g-row-top",
                for (name , items , at) in states {
                    Specimen { key: "{name}", name,
                        Posed { items, at }
                    }
                }
            }
        }
    }
}

/// The pop-up button at every size and kind.
#[component]
fn PopUps() -> Element {
    let mut chosen = use_signal(|| 1u8);
    let mut pulled = use_signal(|| "nothing yet".to_string());
    let options = || {
        vec![
            MenuItem::new(0u8, "Small"),
            MenuItem::new(1u8, "Medium"),
            MenuItem::new(2u8, "Large"),
            MenuItem::Separator,
            MenuItem::new(3u8, "Extra large").with_availability(Availability::Disabled),
        ]
    };
    rsx! {
        Section {
            title: "PopUpButton",
            note: "A pop-up shows the chosen item and marks it in its menu; a pull-down keeps a fixed title and marks nothing; an overflow is a pull-down drawn as the ⋯ alone (Finder's Action button), for a row's or toolbar's extra commands. Letters typed while the button holds the keyboard choose the next item that starts with them; Down opens the menu.",
            div { class: "g-row g-row-top",
                for size in [ControlSize::Mini, ControlSize::Small, ControlSize::Regular, ControlSize::Large] {
                    Specimen { key: "{size:?}", name: format!("Pop-up, {size:?}"),
                        PopUpButton::<u8> { items: options(), value: Some(chosen()), size, onpick: move |value| chosen.set(value) }
                    }
                }
                Specimen { name: "Pull-down",
                    PopUpButton::<u8> {
                        kind: PopUpKind::PullDown,
                        items: options(),
                        title: Some("Actions".to_string()),
                        onpick: move |value| pulled.set(format!("item {value}")),
                    }
                }
                Specimen { name: "Overflow", code: "PopUpKind::Overflow".to_string(),
                    PopUpButton::<u8> {
                        kind: PopUpKind::Overflow,
                        items: options(),
                        title: Some("More".to_string()),
                        onpick: move |value| pulled.set(format!("item {value}")),
                    }
                }
                Specimen { name: "Disabled",
                    PopUpButton::<u8> { items: options(), value: Some(0), availability: Availability::Disabled, onpick: |_| {} }
                }
                Specimen { name: "Busy",
                    PopUpButton::<u8> { items: options(), value: Some(0), availability: Availability::Busy, onpick: |_| {} }
                }
            }
            p { class: "g-note", "Pulled down: {pulled}" }
        }
    }
}
