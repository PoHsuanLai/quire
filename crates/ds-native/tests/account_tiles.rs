//! mailo gaps 2, step 2: the Add account tile on a real Blitz document. It presses like any
//! tile, is never pressed itself, and sits on no plate at rest where an account tile has one.

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::{
    AccountFace, AccountTile, AddAccountTile, Appearance, Colour, Ds, Grain, Hex, Material,
    Provider, SpaceLook, Switch,
};
use ds_native::{Harness, Viewport};
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

/// An account tile and the Add account tile, in the tiles' own grid on a window with no grain,
/// each press logged.
#[allow(non_snake_case)]
fn Tiles() -> Element {
    let mut log = use_signal(Vec::<&str>::new);
    let look = SpaceLook {
        grain: Grain(0),
        ..SpaceLook::default()
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, look,
            div { class: "ds-account-tiles", style: "width:240px; padding:12px",
                AccountTile {
                    account: AccountFace::One {
                        initial: 'P',
                        colour: Colour::Solid(Hex([0x5b, 0x4f, 0xc4])),
                        provider: Provider::Fastmail,
                        address: None,
                    },
                    pressed: Switch::Off,
                    unread: 0,
                    onclick: move |()| log.with_mut(|log| log.push("account")),
                }
                AddAccountTile { title: "Add account…", onclick: move |()| log.with_mut(|log| log.push("add")) }
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
    harness.click(harness.centre(add).expect("the add tile"));
    harness.advance(ms(50));
    assert_eq!(before, "");
    assert_eq!(harness.text_of(".log").as_deref(), Some("add"));
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
    let account = corner(".ds-account-tile:not([*|data-face])");
    let add = corner("[*|data-face=add]");
    assert_ne!(account, ground, "the account tile has a plate");
    assert_eq!(add, ground, "the add tile has none");
}
