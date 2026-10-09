//! The pinned grid on a real Blitz document: the Add account tile presses like any tile, is never
//! pressed itself and sits on no plate at rest where an account tile has one; a tile dragged onto
//! another's place takes it. (The file keeps the name it had when `AccountTile` was the tile.)

use crate::support::probe;

use dioxus::prelude::*;
use ds::components::app::pin_tile::{PinFace, PinStatus};
use ds::components::app::pin_tiles::{PinAdd, PinItem, PinMenu, PinTiles};
use ds::components::content::image_source::ImageSource;
use ds::components::content::provider_mark::{MarkProvider, MarkStyle};
use ds::prelude::*;
use ds::style::tokens::hex::{Colour, Hex};
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use probe::rect;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 320,
    height: 160,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// An account's tile.
fn account(key: char, provider: MarkProvider) -> PinItem<char> {
    PinItem::new(
        key,
        PinFace::Account {
            initial: key,
            colour: Colour::Solid(Hex([0x5b, 0x4f, 0xc4])),
            provider,
            address: None,
        },
    )
}

/// Three accounts and the Add account tile, in the pinned grid on a window with no grain, each
/// press and each new order logged.
#[allow(non_snake_case)]
fn Tiles() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut order = use_signal(|| vec!['P', 'W', 'G']);
    let look = SpaceLook {
        grain: Grain(0),
        ..SpaceLook::default()
    };
    let items: Vec<PinItem<char>> = order()
        .into_iter()
        .map(|key| {
            let item = account(key, MarkProvider::Fastmail);
            match key {
                'W' => item.mark(MarkStyle::Image(ImageSource(
                    "data:image/png;base64,iVBORw0KGgo=".to_string(),
                ))),
                _ => item,
            }
        })
        .collect();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, look,
            div { style: "width:240px; padding:12px",
                PinTiles {
                    label: "Accounts",
                    items,
                    onpick: move |key: char| log.with_mut(|log| log.push(format!("pick:{key}"))),
                    onmenu: move |menu: PinMenu<char>| log.with_mut(|log| log.push(format!("menu:{}", menu.key))),
                    onreorder: move |next: Vec<char>| {
                        log.with_mut(|log| log.push(format!("order:{}", next.iter().collect::<String>())));
                        order.set(next);
                    },
                    add: PinAdd {
                        label: "Add account".to_string(),
                        hint: Some("Add Account".to_string()),
                        onadd: EventHandler::new(move |()| log.with_mut(|log| log.push("add".to_string()))),
                    },
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

/// A tile draws the mark its own item names: the one account with a favicon shows an image, the
/// other two their letter.
#[test]
fn each_tile_draws_the_mark_its_item_names() {
    let mut harness = Harness::new(Tiles, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(50));
    assert_eq!(
        harness.count(".ds-pin-tile .ds-provider[*|data-kind=image]"),
        1
    );
    assert_eq!(
        harness.count(".ds-pin-tile .ds-provider[*|data-kind=letter]"),
        2
    );
    let image = harness
        .centre(".ds-provider[*|data-kind=image]")
        .expect("the favicon");
    let second = harness
        .centre(".ds-pin-tile:nth-child(2)")
        .expect("the second tile");
    assert!(
        (image.x.0 - second.x.0).abs() < 20.0,
        "the favicon is on the second tile"
    );
}

#[test]
fn the_add_tile_presses_and_is_never_pressed() {
    let mut harness = Harness::new(Tiles, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    let add = "[*|data-face=add]";
    assert_eq!(harness.attr(add, "aria-pressed"), None);
    assert_eq!(
        harness.attr(add, "aria-label").as_deref(),
        Some("Add account")
    );
    let before = harness.text_of(".log").unwrap_or_default();
    harness.send(Input::click(harness.centre(add).expect("the add tile")));
    harness.advance(ms(50));
    assert_eq!(before, "");
    assert_eq!(harness.text_of(".log").as_deref(), Some("add"));
}

/// A tile dragged onto another's place takes it, and the drag is not also a pick; a press that
/// never moves is a pick.
#[test]
fn a_dragged_tile_takes_the_place_it_is_dropped_on() {
    let mut harness = Harness::new(Tiles, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(50));
    let tiles = ".ds-pin-tile[*|data-face=account]";
    let first = harness.centre(tiles).expect("the first tile");
    harness.send(Input::click(first));
    harness.advance(ms(50));
    assert_eq!(harness.text_of(".log").as_deref(), Some("pick:P"));
    let third = {
        let all = rect(&harness, ".ds-pin-tiles");
        let tile = rect(&harness, tiles);
        // The third of four columns: two tile pitches along from the first.
        let pitch = (all.size.width.0 - tile.size.width.0) / 3.0;
        Point {
            x: Px(first.x.0 + 2.0 * pitch),
            y: first.y,
        }
    };
    harness.send(Input::pointer_move(first));
    harness.send(Input::pointer_down(first));
    harness.advance(ms(50));
    for part in [0.25f32, 0.5, 0.75, 1.0] {
        harness.send(Input::pointer_move(Point {
            x: Px(first.x.0 + (third.x.0 - first.x.0) * part),
            y: first.y,
        }));
        harness.advance(ms(20));
    }
    assert_eq!(
        harness.count(".ds-pin-tile[*|data-drop=target]"),
        1,
        "the drop line shows where it would land"
    );
    assert_eq!(harness.count(".ds-pin-tile[*|data-drag=source]"), 1);
    harness.send(Input::pointer_up(third));
    harness.advance(ms(50));
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some("pick:P,order:WGP"),
        "{}",
        harness.html()
    );
}

/// At rest the account tile's corner is its plate (`--f-pill-hover`), the add tile's is the
/// window's ground: the add tile's rule outranks the Pin's ground, which comes later.
#[test]
fn the_add_tile_has_no_plate_at_rest() {
    let mut harness = Harness::new(Tiles, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    let frame = harness.render().expect("a frame");
    probe::keep(&frame, "account_tiles");
    // `dy` down from the tile's top edge, at its middle: 3 is inside, -6 the padding above.
    let at = |selector: &str, dy: f32| {
        let tile = rect(&harness, selector);
        let x = (tile.origin.x.0 + tile.size.width.0 / 2.0) as u32;
        let y = (tile.origin.y.0 + dy) as u32;
        frame.get_pixel(x, y).0
    };
    let corner = |selector: &str| at(selector, 3.0);
    let ground = at("[*|data-face=add]", -6.0);
    let account = corner(".ds-pin-tile:not([*|data-face=add])");
    let add = corner("[*|data-face=add]");
    assert_ne!(account, ground, "the account tile has a plate");
    assert_eq!(add, ground, "the add tile has none");
}

static RUNNING: GlobalSignal<bool> = Signal::global(|| true);

/// Four accounts, one of each status: A needs attention, B is working, C is working with no
/// operation running, D is quiet. Each pick, status press and new order is logged.
#[allow(non_snake_case)]
fn Statuses() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let operation = if RUNNING() {
        Operation::Running(PendingToken::start())
    } else {
        Operation::Idle
    };
    let items: Vec<PinItem<char>> = ['A', 'B', 'C', 'D']
        .into_iter()
        .map(|key| {
            let item = account(key, MarkProvider::Fastmail).unread(u32::from(key == 'A') * 4);
            match key {
                'A' => item.status(PinStatus::Attention {
                    why: "Password rejected".to_owned(),
                }),
                'B' => item.status(PinStatus::Busy(operation)),
                'C' => item.status(PinStatus::Busy(Operation::Idle)),
                _ => item,
            }
        })
        .collect();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "width:240px; padding:12px",
                PinTiles {
                    label: "Accounts",
                    items,
                    onpick: move |key: char| log.with_mut(|log| log.push(format!("pick:{key}"))),
                    onreorder: move |next: Vec<char>| log.with_mut(|log| log.push(format!("order:{}", next.iter().collect::<String>()))),
                    onstatus: move |key: char| log.with_mut(|log| log.push(format!("status:{key}"))),
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

fn statuses() -> Harness {
    let mut harness = Harness::new(
        Statuses,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    harness.advance(ms(100));
    harness
}

const ATTENTION: &str = ".ds-pin-tile[*|data-status=attention] .ds-pin-tile-status";

/// The warning sits in the top-left corner, named by its reason; the unread count keeps the top
/// right; a quiet tile has no mark.
#[test]
fn an_attention_mark_sits_top_left_named_by_its_reason() {
    let harness = statuses();
    assert_eq!(harness.count(".ds-pin-tile-status"), 3);
    assert_eq!(
        harness.attr(ATTENTION, "aria-label").as_deref(),
        Some("Password rejected")
    );
    assert_eq!(harness.attr(ATTENTION, "role").as_deref(), Some("button"));
    assert_eq!(harness.count(&format!("{ATTENTION} .ds-ic")), 1);
    let tile = rect(&harness, ".ds-pin-tile[*|data-status=attention]");
    let mark = rect(&harness, ATTENTION);
    let badge = rect(&harness, ".ds-pin-tile[*|data-status=attention] .ds-badge");
    let centre = |r: ds::base::geometry::units::Rect| r.origin.x.0 + r.size.width.0 / 2.0;
    assert!(
        centre(mark) < tile.origin.x.0 + tile.size.width.0 / 2.0,
        "the mark is on the left half: {mark:?} {tile:?}"
    );
    assert!(
        centre(badge) > tile.origin.x.0 + tile.size.width.0 / 2.0,
        "the count is on the right half: {badge:?}"
    );
    assert!(
        mark.origin.x.0 + mark.size.width.0 <= badge.origin.x.0,
        "the two never overlap: {mark:?} {badge:?}"
    );
    assert_eq!(
        harness.count(".ds-pin-tile:not([*|data-status]) .ds-pin-tile-status"),
        0
    );
}

/// Pressing the warning reports its tile once and is neither a pick nor the start of a drag;
/// pressing the tile elsewhere still picks.
#[test]
fn pressing_the_warning_is_its_own_press_and_never_a_pick() {
    let mut harness = statuses();
    let at = harness.centre(ATTENTION).expect("the warning");
    harness.send(Input::click(at));
    harness.advance(ms(50));
    assert_eq!(harness.text_of(".log").as_deref(), Some("status:A"));
    // A press that drags from the warning moves nothing.
    harness.send(Input::pointer_move(at));
    harness.send(Input::pointer_down(at));
    harness.advance(ms(50));
    harness.send(Input::pointer_move(Point {
        x: Px(at.x.0 + 80.0),
        y: at.y,
    }));
    harness.advance(ms(50));
    assert_eq!(harness.count(".ds-pin-tile[*|data-drag=source]"), 0);
    harness.send(Input::pointer_up(at));
    harness.advance(ms(50));
    let tile = rect(&harness, ".ds-pin-tile[*|data-status=attention]");
    let body = Point {
        x: Px(tile.origin.x.0 + tile.size.width.0 * 0.6),
        y: Px(tile.origin.y.0 + tile.size.height.0 * 0.6),
    };
    harness.send(Input::click(body));
    harness.advance(ms(50));
    let log = harness.text_of(".log").unwrap_or_default();
    assert!(log.ends_with("pick:A") && !log.contains("order"), "{log}");
    assert_eq!(log.matches("status:").count(), 1);
}

/// A busy tile's Mini spinner turns only while its operation runs (R4).
#[test]
fn a_busy_mark_turns_only_while_its_operation_runs() {
    let mut harness = statuses();
    let spin = |n: usize| format!(".ds-pin-tile:nth-child({n}) .ds-pin-tile-status .ds-progress");
    assert_eq!(harness.attr(&spin(2), "data-size").as_deref(), Some("mini"));
    assert_eq!(
        harness.attr(&spin(2), "data-pending").as_deref(),
        Some("step")
    );
    assert_eq!(
        harness.attr(&spin(3), "data-pending").as_deref(),
        Some("idle"),
        "no running operation, no turning spinner"
    );
    harness.within(|| *RUNNING.write() = false);
    harness.advance(ms(50));
    assert_eq!(
        harness.attr(&spin(2), "data-pending").as_deref(),
        Some("idle")
    );
}

/// One account's tile in a grid `width` px wide: 160 is the content of a sidebar at its least
/// (180), 212 that of a 232 px sidebar.
fn one_tile(width: u32) -> Harness {
    WIDTH.with(|cell| cell.set(width));
    let mut harness = Harness::new(OneTile, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(100));
    harness
}

thread_local! {
    static WIDTH: std::cell::Cell<u32> = const { std::cell::Cell::new(212) };
}

#[allow(non_snake_case)]
fn OneTile() -> Element {
    let width = WIDTH.with(std::cell::Cell::get);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "width:{width}px",
                PinTiles {
                    label: "Accounts",
                    items: vec![account('A', MarkProvider::Fastmail)],
                    onpick: move |_: char| {},
                    onreorder: move |_: Vec<char>| {},
                }
            }
        }
    }
}

/// The provider's mark sits on the face's corner at any tile width: as far from the avatar's
/// centre in a narrow tile as at the usual width, where it is where it always was (7 px in from
/// the tile). A fixed inset from the tile instead slid it over the letter as the tile shrank.
#[test]
fn the_provider_mark_keeps_its_place_on_the_face_at_any_tile_width() {
    let from_centre = |width: u32| {
        let harness = one_tile(width);
        let face = rect(&harness, ".ds-pin-tile .ds-avatar");
        let mark = rect(&harness, ".ds-pin-tile .ds-provider");
        (
            mark.origin.x.0 - (face.origin.x.0 + face.size.width.0 / 2.0),
            mark.origin.y.0 - (face.origin.y.0 + face.size.height.0 / 2.0),
        )
    };
    let (usual, narrow) = (from_centre(212), from_centre(160));
    assert!(
        (usual.0 - narrow.0).abs() <= 0.5 && (usual.1 - narrow.1).abs() <= 0.5,
        "the mark moved on the face: {usual:?} from its centre at 212 px, {narrow:?} at 160 px"
    );
    let harness = one_tile(212);
    let tile = rect(&harness, ".ds-pin-tile");
    let mark = rect(&harness, ".ds-pin-tile .ds-provider");
    let inset = tile.origin.x.0 + tile.size.width.0 - (mark.origin.x.0 + mark.size.width.0);
    assert!(
        (inset - 7.0).abs() <= 1.0,
        "the mark moved at the usual width: {inset}px in from the tile"
    );
}

/// A secondary press asks for that tile's menu and is never a pick; a primary press still picks
/// and asks for no menu.
#[test]
fn a_secondary_press_asks_for_the_tile_s_menu_and_picks_nothing() {
    let mut harness = Harness::new(Tiles, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(50));
    let second = harness
        .centre(".ds-pin-tile:nth-child(2)")
        .expect("the second tile");
    harness.send(Input::press(
        second,
        ds::base::press::PointerButton::Secondary,
    ));
    harness.advance(ms(50));
    assert_eq!(harness.text_of(".log").as_deref(), Some("menu:W"));
    harness.send(Input::click(second));
    harness.advance(ms(50));
    assert_eq!(harness.text_of(".log").as_deref(), Some("menu:W,pick:W"));
}
