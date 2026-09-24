//! The launcher gaps on a real Blitz document (sill FINDINGS Q40-Q42, Q44): the palette
//! embedded in a surface of its own, its selection and the selected row's rect reported to the
//! caller, a field given the keyboard back after a menu took it, and an app icon in a row.

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::{
    Anchor, Appearance, Availability, CommandPalette, CommandPaletteHost, Ds, ExternalIcon,
    FocusRequest, IconSize, IconSource, IconUrl, Key, Material, Menu, MenuEntry, MenuKind,
    PaletteEntrance, Px, Rect, Tile, Trail, use_focus_request,
};
use ds_native::{Harness, Viewport};
use probe::rect;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 480,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn item(value: u8, title: &str) -> MenuEntry<u8> {
    MenuEntry::Item {
        value,
        title: title.to_string(),
        detail: None,
        tile: None,
        trail: Trail::None,
        check: None,
        availability: Availability::Enabled,
    }
}

fn groups() -> Vec<(String, Vec<MenuEntry<u8>>)> {
    vec![(
        "Applications".to_string(),
        vec![item(1, "Files"), item(2, "Firefox"), item(3, "Terminal")],
    )]
}

/// A rect as the log writes it, to the pixel.
fn show(rect: Rect) -> String {
    format!(
        "{}x{}@{},{}",
        rect.size.width.0.round(),
        rect.size.height.0.round(),
        rect.origin.x.0.round(),
        rect.origin.y.0.round()
    )
}

/// How the page under test drives the palette's selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Selection {
    /// The palette's own.
    Own,
    /// The page's, following every request.
    Followed,
    /// The page's, ignoring every request.
    Pinned,
}

/// Whether the menu's close hands the keyboard back to the field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GiveBack {
    Yes,
    No,
}

/// A launcher panel: a 600 x 400 container holding the palette as a surface, with the page's
/// log of what the palette reported, and Tab opening an actions menu at the selected row's rect
/// that hands the keyboard back when it closes (when `give_back` says so).
#[component]
fn Panel(selection: Selection, give_back: GiveBack) -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut note = move |line: String| log.with_mut(|log| log.push(line));
    let mut row = use_signal(|| None::<Rect>);
    let mut actions = use_signal(|| false);
    let mut chosen = use_signal(|| 0usize);
    let focus: FocusRequest = use_focus_request();
    let selected = match selection {
        Selection::Own => None,
        Selection::Followed | Selection::Pinned => Some(chosen()),
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { class: "panel", style: "width:600px; height:400px",
                CommandPalette::<u8> {
                    label: "Launch".to_string(),
                    placeholder: "Search".to_string(),
                    query: String::new(),
                    tokens: Vec::new(),
                    groups: groups(),
                    empty: "Nothing".to_string(),
                    oninput: move |_| {},
                    onpick: move |value: u8| note(format!("pick:{value}")),
                    onclose: move |()| note("close".to_string()),
                    host: CommandPaletteHost::Surface,
                    entrance: PaletteEntrance::CmdkIn,
                    id: "launcher-card".to_string(),
                    focus,
                    selected,
                    on_select: move |index: usize| {
                        note(format!("select:{index}"));
                        if selection == Selection::Followed {
                            chosen.set(index);
                        }
                    },
                    on_select_rect: move |rect: Rect| {
                        note(format!("rect:{}", show(rect)));
                        row.set(Some(rect));
                    },
                    onkey: move |event: KeyboardEvent| {
                        if event.key() == dioxus::prelude::Key::Tab {
                            note("key:Tab".to_string());
                            actions.set(true);
                        }
                    },
                }
            }
            p { class: "log", {log().join(",")} }
            if actions() {
                Menu::<u8> {
                    kind: MenuKind::Slim,
                    anchor: Anchor::Rect(row().unwrap_or_default()),
                    entries: vec![item(9, "Quit")],
                    onpick: move |_| {},
                    onclose: move |()| {
                        actions.set(false);
                        if give_back == GiveBack::Yes {
                            focus.request();
                        }
                    },
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn OwnSelection() -> Element {
    rsx! { Panel { selection: Selection::Own, give_back: GiveBack::Yes } }
}

#[allow(non_snake_case)]
fn FollowedSelection() -> Element {
    rsx! { Panel { selection: Selection::Followed, give_back: GiveBack::Yes } }
}

#[allow(non_snake_case)]
fn PinnedSelection() -> Element {
    rsx! { Panel { selection: Selection::Pinned, give_back: GiveBack::Yes } }
}

#[allow(non_snake_case)]
fn KeptFocus() -> Element {
    rsx! { Panel { selection: Selection::Own, give_back: GiveBack::No } }
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

/// The `n`th row (from 1) of the palette's list: its group's header is the list's first child.
fn nth_row(n: usize) -> String {
    format!("#launcher-card .ds-menu-item:nth-child({})", n + 1)
}

/// Let the palette mount, measure and report.
fn settle_in(harness: &mut Harness) {
    harness.advance(ms(120));
}

/// Q40: embedded, the palette draws no scrim, its card spans the container's width from its top
/// and, since the macOS polish pass, is only as tall as its content (at most the container's),
/// carries the id a blur region names, and enters with the entrance asked for.
#[test]
fn an_embedded_palette_fills_its_container_with_no_scrim() {
    let mut harness = Harness::new(OwnSelection, VIEW);
    settle_in(&mut harness);
    assert_eq!(harness.count(".ds-palette-wrap"), 0, "no scrim");
    assert_eq!(harness.count(".ds-overlay[*|data-layer=palette]"), 0);
    let card = rect(&harness, "#launcher-card");
    let panel = rect(&harness, ".panel");
    assert_eq!(
        card.origin, panel.origin,
        "the card starts at its container's corner"
    );
    assert_eq!(card.size.width, panel.size.width, "and spans its width");
    assert!(
        card.size.height.0 < panel.size.height.0,
        "and is as tall as its content: {card:?}"
    );
    assert_eq!(
        harness.attr("#launcher-card", "data-entrance").as_deref(),
        Some("cmdk-in")
    );
    assert_eq!(
        harness.attr("#launcher-card", "data-host").as_deref(),
        Some("surface")
    );
}

/// Q41: an uncontrolled palette reports its first selection and the selected row's rect, then
/// every move, by key or by pointer, with the rect of the row it moved to.
#[test]
fn the_palette_reports_its_selection_and_the_rows_rect() {
    let mut harness = Harness::new(OwnSelection, VIEW);
    settle_in(&mut harness);
    let first = show(rect(&harness, &nth_row(1)));
    assert_eq!(log(&harness), format!("select:0,rect:{first}"));
    harness.key(Key::Down);
    settle_in(&mut harness);
    let second = show(rect(&harness, &nth_row(2)));
    assert_eq!(
        log(&harness),
        format!("select:0,rect:{first},select:1,rect:{second}")
    );
    let third = rect(&harness, &nth_row(3));
    harness.pointer_move(harness.centre(&nth_row(3)).expect("the third row"));
    settle_in(&mut harness);
    assert_eq!(
        log(&harness),
        format!(
            "select:0,rect:{first},select:1,rect:{second},select:2,rect:{}",
            show(third)
        )
    );
    assert_eq!(
        harness.attr(&nth_row(3), "aria-selected").as_deref(),
        Some("true"),
        "the pointer moved the selection"
    );
}

/// Q41: controlled, the palette shows the caller's selection and only asks to move it.
#[test]
fn a_controlled_selection_moves_only_when_the_caller_moves_it() {
    let mut followed = Harness::new(FollowedSelection, VIEW);
    settle_in(&mut followed);
    followed.key(Key::Down);
    settle_in(&mut followed);
    assert!(log(&followed).starts_with("rect:"), "{}", log(&followed));
    assert!(log(&followed).contains(",select:1,"), "{}", log(&followed));
    assert_eq!(
        followed.attr(&nth_row(2), "aria-selected").as_deref(),
        Some("true")
    );
    let mut pinned = Harness::new(PinnedSelection, VIEW);
    settle_in(&mut pinned);
    pinned.key(Key::Down);
    settle_in(&mut pinned);
    assert!(log(&pinned).ends_with(",select:1"), "{}", log(&pinned));
    assert_eq!(
        pinned.attr(&nth_row(1), "aria-selected").as_deref(),
        Some("true"),
        "the caller kept the first row"
    );
}

/// Opens the actions menu with Tab (a key the palette hands on), checks the menu is at the
/// selected row and has the keyboard, and closes it with Escape.
fn open_and_close_actions(harness: &mut Harness) {
    assert!(
        harness.is_focused("#launcher-card .ds-input"),
        "the field starts focused"
    );
    harness.key(Key::Tab);
    settle_in(harness);
    assert!(log(harness).contains("key:Tab"), "{}", log(harness));
    assert!(
        harness.is_focused(".ds-popover.ds-menu"),
        "the menu took the keyboard"
    );
    let menu = rect(harness, ".ds-popover.ds-menu");
    let row = rect(harness, &nth_row(1));
    assert!(
        (menu.origin.y.0 - (row.origin.y.0 + row.size.height.0 + 6.0)).abs() < 1.0,
        "the menu hangs 6 px under the selected row: {menu:?} {row:?}"
    );
    harness.key(Key::Escape);
    harness.advance(ms(400));
    assert_eq!(harness.count(".ds-popover.ds-menu"), 0, "the menu closed");
}

/// Q44: the field gets the keyboard back when the menu closes and asks for it, without a
/// remount (the card is still at rest, not replaying its entrance); without the request it
/// does not.
#[test]
fn a_focus_request_gives_the_field_the_keyboard_back() {
    let mut harness = Harness::new(OwnSelection, VIEW);
    harness.advance(ms(700));
    assert_eq!(
        harness.attr("#launcher-card", "data-presence").as_deref(),
        Some("present")
    );
    open_and_close_actions(&mut harness);
    assert!(
        harness.is_focused("#launcher-card .ds-input"),
        "focus came back"
    );
    assert_eq!(
        harness.attr("#launcher-card", "data-presence").as_deref(),
        Some("present"),
        "the palette was not remounted"
    );
    let mut kept = Harness::new(KeptFocus, VIEW);
    kept.advance(ms(700));
    open_and_close_actions(&mut kept);
    assert!(
        !kept.is_focused("#launcher-card .ds-input"),
        "without the request the field stays unfocused"
    );
}

/// A 4 x 4 red PNG, as an app icon.
fn red_png() -> Vec<u8> {
    let image = image::RgbaImage::from_pixel(4, 4, image::Rgba([230, 20, 20, 255]));
    let mut bytes = std::io::Cursor::new(Vec::new());
    image
        .write_to(&mut bytes, image::ImageFormat::Png)
        .expect("a PNG");
    bytes.into_inner()
}

#[allow(non_snake_case)]
fn AppIconRow() -> Element {
    let icon = IconSource::Image(ExternalIcon {
        url: IconUrl::png(&red_png()),
        size: IconSize::Bar,
    });
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "width:600px; height:400px",
                CommandPalette::<u8> {
                    label: "Launch".to_string(),
                    placeholder: "Search".to_string(),
                    query: String::new(),
                    tokens: Vec::new(),
                    groups: vec![(
                        "Applications".to_string(),
                        vec![MenuEntry::Item {
                            value: 1,
                            title: "Firefox".to_string(),
                            detail: None,
                            tile: Some(Tile::Source(icon)),
                            trail: Trail::None,
                            check: None,
                            availability: Availability::Enabled,
                        }],
                    )],
                    empty: "Nothing".to_string(),
                    oninput: move |_| {},
                    onpick: move |_| {},
                    onclose: move |()| {},
                    host: CommandPaletteHost::Surface,
                }
            }
        }
    }
}

/// Q42: an app icon in a row fills the row's tile, drawn as it is (red), on no plate.
#[test]
fn an_app_icon_fills_its_rows_tile() {
    let mut harness = Harness::new(AppIconRow, VIEW);
    harness.advance(ms(700));
    let tile = rect(&harness, ".ds-menu-tile[*|data-tile=image]");
    let icon = rect(&harness, ".ds-menu-tile > .ds-ext-icon");
    assert_eq!(tile.size.width, Px(34.0));
    assert_eq!(icon, tile, "the icon fills the tile");
    let frame = harness.render().expect("a frame");
    let centre = frame.get_pixel(
        (icon.origin.x.0 + icon.size.width.0 / 2.0) as u32,
        (icon.origin.y.0 + icon.size.height.0 / 2.0) as u32,
    );
    assert!(
        centre[0] > 200 && centre[1] < 60 && centre[2] < 60,
        "drawn as it is: {centre:?}"
    );
}
