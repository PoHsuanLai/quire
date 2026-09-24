//! The palette follow-ups on a real Blitz document (sill FINDINGS Q60, Q61, Q63): the selected
//! row's rect on a surface that is laid out only after the palette mounted, a key the caller
//! takes and keeps from the document, and a palette kept mounted while hidden that replays its
//! entrance each time it is shown.

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::{
    Appearance, Availability, CommandPalette, CommandPaletteHost, Ds, Key, Material, MenuEntry,
    PaletteEntrance, Rect, Retain, Shown, Trail,
};
use ds_native::{Harness, Viewport};
use image::RgbaImage;
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

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

/// The `n`th row (from 1) of the palette's list: its group's header is the list's first child.
fn nth_row(n: usize) -> String {
    format!("#card .ds-menu-item:nth-child({})", n + 1)
}

/// A launcher panel logging the palette's selection and row rects; typing into the field
/// gives the field an operator token, which moves the list down without moving the selection.
#[allow(non_snake_case)]
fn Rows() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut query = use_signal(String::new);
    let tokens = if query().is_empty() {
        Vec::new()
    } else {
        vec!["in:apps".to_string()]
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "width:600px; height:400px",
                CommandPalette::<u8> {
                    label: "Launch".to_string(),
                    placeholder: "Search".to_string(),
                    query: query(),
                    tokens,
                    groups: groups(),
                    empty: "Nothing".to_string(),
                    oninput: move |text: String| query.set(text),
                    onpick: move |_| {},
                    onclose: move |()| {},
                    host: CommandPaletteHost::Surface,
                    id: "card".to_string(),
                    on_select: move |index: usize| log.with_mut(|log| log.push(format!("select:{index}"))),
                    on_select_rect: move |rect: Rect| log.with_mut(|log| log.push(format!("rect:{}", show(rect)))),
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

/// Q60: built before its surface is laid out, the palette reports no rect while every read is
/// 0 x 0, then its first row's real rect a few frames after the surface is laid out; the rect
/// follows the selection, and follows the row when the results above it move it.
#[test]
fn a_palette_on_a_fresh_surface_reports_its_first_row_once_laid_out() {
    let mut harness = Harness::unmapped(Rows, VIEW);
    harness.advance(ms(150));
    assert_eq!(
        log(&harness),
        "select:0",
        "no rect is reported before layout"
    );
    harness.map();
    harness.advance(ms(100));
    let first = rect(&harness, &nth_row(1));
    assert!(first.size.width.0 > 0.0 && first.size.height.0 > 0.0);
    assert_eq!(log(&harness), format!("select:0,rect:{}", show(first)));
    harness.key(Key::Down);
    harness.advance(ms(100));
    let second = rect(&harness, &nth_row(2));
    assert!(
        second.origin.y.0 > first.origin.y.0,
        "the selection moved down"
    );
    assert!(
        log(&harness).ends_with(&format!(",select:1,rect:{}", show(second))),
        "{}",
        log(&harness)
    );
    harness.key(Key::Up);
    harness.advance(ms(100));
    harness.key(Key::Char('f'));
    harness.advance(ms(100));
    let moved = rect(&harness, &nth_row(1));
    assert!(
        moved.origin.y.0 > first.origin.y.0,
        "the token row moved the list down: {moved:?} {first:?}"
    );
    assert!(
        log(&harness).ends_with(&format!(",rect:{}", show(moved))),
        "the same row, moved by new results, is reported again: {}",
        log(&harness)
    );
}

/// Whether the page's `onkey` keeps Tab from the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TakesTab {
    Consumes,
    Passes,
}

/// A palette whose caller reads Tab from `onkey` (an actions key), with a button after it for
/// the document's own Tab to move the focus to.
#[component]
fn TabPage(takes: TakesTab) -> Element {
    let mut tabs = use_signal(|| 0u32);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "width:600px; height:300px",
                CommandPalette::<u8> {
                    label: "Launch".to_string(),
                    placeholder: "Search".to_string(),
                    query: String::new(),
                    tokens: Vec::new(),
                    groups: groups(),
                    empty: "Nothing".to_string(),
                    oninput: move |_| {},
                    onpick: move |_| {},
                    onclose: move |()| {},
                    host: CommandPaletteHost::Surface,
                    id: "card".to_string(),
                    onkey: move |event: KeyboardEvent| {
                        if event.key() == dioxus::prelude::Key::Tab {
                            tabs += 1;
                            if takes == TakesTab::Consumes {
                                event.prevent_default();
                            }
                        }
                    },
                }
            }
            button { class: "after", "After" }
            p { class: "tabs", "{tabs}" }
        }
    }
}

#[allow(non_snake_case)]
fn TabConsumed() -> Element {
    rsx! { TabPage { takes: TakesTab::Consumes } }
}

#[allow(non_snake_case)]
fn TabPassed() -> Element {
    rsx! { TabPage { takes: TakesTab::Passes } }
}

/// Press Tab in the palette's field; whether the field still has the keyboard after.
fn tab_keeps_the_field(page: fn() -> Element) -> bool {
    let mut harness = Harness::new(page, VIEW);
    harness.advance(ms(200));
    assert!(
        harness.is_focused("#card .ds-input"),
        "the field starts focused"
    );
    harness.key(Key::Tab);
    harness.advance(ms(50));
    assert_eq!(
        harness.text_of(".tabs").as_deref(),
        Some("1"),
        "the caller heard Tab"
    );
    harness.is_focused("#card .ds-input")
}

/// Q61: a Tab the caller takes (`prevent_default` on the event `onkey` hands it) leaves the
/// field focused; one it only listens to still moves the focus, as the document's default.
#[test]
fn a_key_the_caller_takes_does_not_move_the_focus() {
    assert!(
        tab_keeps_the_field(TabConsumed),
        "the taken Tab kept the field"
    );
    assert!(
        !tab_keeps_the_field(TabPassed),
        "an untaken Tab moves the focus"
    );
}

/// A launcher that keeps its palette mounted and shows it with a button (the shell's toggle),
/// typing into its field on `f`.
#[component]
fn KeptPage(retain: Retain) -> Element {
    let mut shown = use_signal(|| Shown::Hidden);
    let mut query = use_signal(String::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            button {
                class: "toggle",
                style: "position:absolute; left:640px; top:420px; width:60px; height:40px",
                onclick: move |_| {
                    let next = match shown() {
                        Shown::Visible => Shown::Hidden,
                        Shown::Hidden => Shown::Visible,
                    };
                    shown.set(next);
                },
                "Toggle"
            }
            div { style: "width:560px; height:360px; margin:20px",
                CommandPalette::<u8> {
                    label: "Launch".to_string(),
                    placeholder: "Search".to_string(),
                    query: query(),
                    tokens: Vec::new(),
                    groups: groups(),
                    empty: "Nothing".to_string(),
                    oninput: move |text: String| query.set(text),
                    onpick: move |_| {},
                    onclose: move |()| shown.set(Shown::Hidden),
                    host: CommandPaletteHost::Surface,
                    entrance: PaletteEntrance::CmdkIn,
                    id: "card".to_string(),
                    shown: shown(),
                    retain,
                }
            }
            p { class: "query", "[{query}]" }
        }
    }
}

#[allow(non_snake_case)]
fn KeptFresh() -> Element {
    rsx! { KeptPage { retain: Retain::Nothing } }
}

#[allow(non_snake_case)]
fn KeptQuery() -> Element {
    rsx! { KeptPage { retain: Retain::Query } }
}

fn toggle(harness: &mut Harness) {
    let at = harness.centre(".toggle").expect("the toggle");
    harness.click(at);
}

/// How many pixels inside `area` differ between two frames.
fn differing(a: &RgbaImage, b: &RgbaImage, area: Rect) -> usize {
    probe::pixels(a, area, 0.0)
        .into_iter()
        .zip(probe::pixels(b, area, 0.0))
        .filter(|(a, b)| a != b)
        .count()
}

/// Where the card's container is: the card starts at its corner and spans its width, as tall as
/// its content.
const CARD: Rect = Rect {
    origin: ds::Point {
        x: ds::Px(20.0),
        y: ds::Px(20.0),
    },
    size: ds::Size {
        width: ds::Px(560.0),
        height: ds::Px(360.0),
    },
};

/// Q63: hidden, the palette lays out and paints nothing; two shows 300 ms apart each play the
/// entrance (a frame 40 ms into it differs from the settled card), each gives the field the
/// keyboard, and a hide leaves the frame as it was before the first show.
#[test]
fn a_kept_palette_replays_its_entrance_on_every_show() {
    let mut harness = Harness::new(KeptFresh, VIEW);
    harness.advance(ms(100));
    let laid = harness
        .rect("#card")
        .map(|card| (card.size.width.0, card.size.height.0));
    assert!(
        matches!(laid, None | Some((0.0, 0.0))),
        "hidden lays out nothing: {laid:?}"
    );
    let hidden = harness.render().expect("a frame");
    assert!(
        !harness.is_focused("#card .ds-input"),
        "hidden takes no keyboard"
    );

    toggle(&mut harness);
    harness.advance(ms(40));
    let first_mid = harness.render().expect("a frame");
    assert!(
        harness.is_focused("#card .ds-input"),
        "shown, it takes the keyboard"
    );
    harness.key(Key::Char('f'));
    harness.advance(ms(110));
    assert_eq!(harness.text_of(".query").as_deref(), Some("[f]"));
    toggle(&mut harness);
    harness.advance(ms(150));
    toggle(&mut harness);
    harness.advance(ms(40));
    let second_mid = harness.render().expect("a frame");
    assert!(
        harness.is_focused("#card .ds-input"),
        "shown again, it has the keyboard"
    );
    assert_eq!(
        harness.text_of(".query").as_deref(),
        Some("[]"),
        "shown again, it starts from an empty query"
    );
    harness.advance(ms(900));
    // Since the macOS polish pass the card is as tall as its content, up to its container.
    let card = rect(&harness, "#card");
    assert_eq!(
        (card.origin, card.size.width),
        (CARD.origin, CARD.size.width),
        "the card is laid out when shown"
    );
    assert!(
        card.size.height.0 > 100.0 && card.size.height.0 <= CARD.size.height.0,
        "and as tall as its content: {card:?}"
    );
    assert_eq!(
        harness.attr("#card", "data-presence").as_deref(),
        Some("present")
    );
    let settled = harness.render().expect("a frame");
    probe::keep(&first_mid, "kept-first-mid");
    probe::keep(&second_mid, "kept-second-mid");
    probe::keep(&settled, "kept-settled");
    let area = card.size.width.0 * card.size.height.0;
    for (name, mid) in [("first", &first_mid), ("second", &second_mid)] {
        let changed = differing(mid, &settled, card);
        assert!(
            changed as f32 > area * 0.1,
            "the {name} show is mid-entrance 40 ms in: {changed} pixels differ"
        );
    }
    assert!(
        differing(&hidden, &settled, card) as f32 > area * 0.5,
        "the shown card is painted"
    );
    toggle(&mut harness);
    harness.advance(ms(100));
    let hidden_again = harness.render().expect("a frame");
    assert_eq!(
        differing(&hidden, &hidden_again, CARD),
        0,
        "hidden again, nothing of the palette is painted"
    );
}

/// Q63 with `Retain::Query`: shown again, the palette keeps the query it had.
#[test]
fn a_kept_palette_can_retain_its_query() {
    let mut harness = Harness::new(KeptQuery, VIEW);
    harness.advance(ms(100));
    toggle(&mut harness);
    harness.advance(ms(100));
    harness.key(Key::Char('f'));
    harness.advance(ms(50));
    toggle(&mut harness);
    harness.advance(ms(100));
    toggle(&mut harness);
    harness.advance(ms(100));
    assert_eq!(harness.text_of(".query").as_deref(), Some("[f]"));
    assert!(harness.is_focused("#card .ds-input"));
}
