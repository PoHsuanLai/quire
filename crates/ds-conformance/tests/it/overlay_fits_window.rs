//! A menu or a palette taller than a short window is capped to the window and scrolls inside
//! it, on a real Blitz document: a floating surface stays 8 px inside the window, as an
//! `NSMenu` does.

use dioxus::prelude::*;
use ds::components::menus::palette::palette_group::{PaletteGroup, PaletteRow};
use ds::host::measure::Anchor;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Query, Viewport};
use std::time::Duration;

const WIDTH: u32 = 480;
const HEIGHT: u32 = 320;
/// How far a floating surface keeps from the window's edge.
const MARGIN: f32 = 8.0;
const ROWS: u8 = 40;

fn short(scale_percent: u16) -> Viewport {
    Viewport {
        width: WIDTH,
        height: HEIGHT,
        scale_percent,
    }
}

fn settled(app: fn() -> Element, scale_percent: u16) -> Harness {
    let mut harness = Harness::new(
        app,
        HarnessConfig::new(short(scale_percent)).with_clock(Clock::Virtual),
    );
    for _ in 0..10 {
        harness.advance(Duration::from_millis(100));
    }
    harness
}

#[allow(non_snake_case)]
fn LongMenu() -> Element {
    let items = (0..ROWS)
        .map(|value| MenuItem::Item {
            value,
            title: format!("Command {value}"),
            image: None,
            key: None,
            check: None,
            availability: Availability::Enabled,
            hint: None,
            after: AfterPick::Close,
            text: Default::default(),
        })
        .collect::<Vec<_>>();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:100vh" }
            Menu {
                placement: MenuPlacement::Context,
                anchor: Anchor::Point(Point { x: Px(100.0), y: Px(100.0) }),
                items,
                onpick: move |_: u8| {},
                onclose: move |_| {},
            }
        }
    }
}

#[allow(non_snake_case)]
fn LongPalette() -> Element {
    let rows = (0..ROWS)
        .map(|value| PaletteRow::new(value, format!("Command {value}")))
        .collect::<Vec<_>>();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:100vh" }
            CommandPalette::<u8> {
                label: "Commands".to_string(),
                placeholder: "Search".to_string(),
                query: String::new(),
                tokens: Vec::<String>::new(),
                groups: vec![PaletteGroup::list("Commands", rows)],
                empty: "Nothing".to_string(),
                oninput: move |_: String| {},
                onpick: move |_| {},
                onclose: move |()| {},
            }
        }
    }
}

fn inside_window(harness: &Harness, selector: &str, what: &str) {
    let rect = harness
        .rect(selector)
        .unwrap_or_else(|| panic!("{what} is not drawn:\n{}", harness.html()));
    let (top, bottom) = (rect.origin.y.0, rect.origin.y.0 + rect.size.height.0);
    assert!(
        top >= MARGIN - 0.5 && bottom <= HEIGHT as f32 - MARGIN + 0.5,
        "{what} spans {top}..{bottom} in a {HEIGHT} px window"
    );
}

#[test]
fn a_menu_taller_than_the_window_fits_inside_it() {
    for scale in [100, 200] {
        let harness = settled(LongMenu, scale);
        inside_window(&harness, ".ds-menu", "the menu");
    }
}

#[test]
fn the_palette_fits_inside_a_short_window() {
    for scale in [100, 200] {
        let harness = settled(LongPalette, scale);
        inside_window(&harness, ".ds-palette", "the palette");
    }
}
