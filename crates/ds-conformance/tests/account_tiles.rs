//! The pinned grid on a real Blitz document: the Add account tile presses like any tile, is never
//! pressed itself and sits on no plate at rest where an account tile has one; a tile dragged onto
//! another's place takes it. (The file keeps the name it had when `AccountTile` was the tile.)

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::components::app::pin_tile::PinFace;
use ds::components::app::pin_tiles::{PinAdd, PinItem, PinTiles};
use ds::components::content::provider_mark::MarkProvider;
use ds::prelude::*;
use ds::style::tokens::hex::{Colour, Hex};
use ds_harness::{Driver, Harness, Input, Query, Viewport};
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
    PinItem {
        key,
        face: PinFace::Account {
            initial: key,
            colour: Colour::Solid(Hex([0x5b, 0x4f, 0xc4])),
            provider,
            address: None,
        },
        unread: 0,
    }
}

/// Three accounts and the Add account tile, in the pinned grid on a window with no grain, each
/// press and each new order logged.
#[allow(non_snake_case)]
fn Tiles() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut order = use_signal(|| vec!['P', 'W', 'G']);
    let look = SpaceLook {
        ..SpaceLook::default()
    };
    let items: Vec<PinItem<char>> = order()
        .into_iter()
        .map(|key| account(key, MarkProvider::Fastmail))
        .collect();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, look,
            div { style: "width:240px; padding:12px",
                PinTiles {
                    label: "Accounts",
                    items,
                    onpick: move |key: char| log.with_mut(|log| log.push(format!("pick:{key}"))),
                    onreorder: move |next: Vec<char>| {
                        log.with_mut(|log| log.push(format!("order:{}", next.iter().collect::<String>())));
                        order.set(next);
                    },
                    add: PinAdd {
                        label: "Add account".to_string(),
                        hint: Some("Add account…".to_string()),
                        onadd: EventHandler::new(move |()| log.with_mut(|log| log.push("add".to_string()))),
                    },
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

#[test]
fn the_add_tile_presses_and_is_never_pressed() {
    let mut harness = Harness::new(Tiles, VIEW);
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
    let mut harness = Harness::new(Tiles, VIEW);
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
    let mut harness = Harness::new(Tiles, VIEW);
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
