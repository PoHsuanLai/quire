//! ModuleTile on a real Blitz document (sill FINDINGS Q78): a press on the tile toggles the
//! module, a press on its chevron opens the detail and does not toggle, and the keys do the same
//! (Enter or Space on the tile toggles; Enter or Right on the chevron opens the detail).

use dioxus::prelude::*;
use ds::{Appearance, Chevron, Ds, Icon, Key, Material, ModuleState, ModuleTile, Point, TileSpan};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 240,
    scale_percent: 100,
};

/// One Wi-Fi tile with a chevron; the page logs every toggle and every detail request.
#[allow(non_snake_case)]
fn TileApp() -> Element {
    let mut state = use_signal(|| ModuleState::Off);
    let mut log = use_signal(Vec::<&'static str>::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Popover,
            div { style: "width:300px;padding:12px",
                ModuleTile {
                    glyph: Icon::Wifi,
                    title: "Wi-Fi",
                    status: "Home",
                    state: state(),
                    chevron: Chevron::Detail,
                    span: TileSpan::Half,
                    onclick: move |_| {
                        state.set(match state() {
                            ModuleState::On => ModuleState::Off,
                            ModuleState::Off | ModuleState::Busy => ModuleState::On,
                        });
                        log.with_mut(|log| log.push("toggle"));
                    },
                    on_detail: move |_| log.with_mut(|log| log.push("detail")),
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

fn centre(harness: &Harness, selector: &str) -> Point {
    harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is not in the document:\n{}", harness.html()))
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

fn state(harness: &Harness) -> String {
    harness
        .attr(".ds-module-tile", "data-state")
        .unwrap_or_default()
}

const TICK: Duration = Duration::from_millis(50);

#[test]
fn the_chevron_opens_the_detail_and_does_not_toggle() {
    let mut harness = Harness::new(TileApp, VIEW);
    harness.advance(TICK);
    harness.click(centre(&harness, ".ds-module-chevron"));
    harness.advance(TICK);
    assert_eq!(log(&harness), "detail");
    assert_eq!(state(&harness), "off", "the chevron did not toggle");

    harness.click(centre(&harness, ".ds-module-title"));
    harness.advance(TICK);
    assert_eq!(log(&harness), "detail,toggle");
    assert_eq!(state(&harness), "on", "the tile itself toggles");
    assert_eq!(
        harness.attr(".ds-module-tile", "aria-pressed").as_deref(),
        Some("true")
    );
}

#[test]
fn the_keys_toggle_on_the_tile_and_open_on_the_chevron() {
    let mut harness = Harness::new(TileApp, VIEW);
    harness.advance(TICK);
    harness.key(Key::Tab);
    assert!(
        harness.is_focused(".ds-module-tile"),
        "Tab reaches the tile"
    );
    harness.key(Key::Enter);
    harness.advance(TICK);
    assert_eq!(log(&harness), "toggle");
    harness.key(Key::Space);
    harness.advance(TICK);
    assert_eq!(log(&harness), "toggle,toggle");
    assert_eq!(state(&harness), "off");

    harness.key(Key::Tab);
    assert!(
        harness.is_focused(".ds-module-chevron"),
        "Tab reaches the chevron"
    );
    harness.key(Key::Enter);
    harness.key(Key::Right);
    harness.advance(TICK);
    assert_eq!(
        log(&harness),
        "toggle,toggle,detail,detail",
        "Enter and Right open the detail, and neither toggles"
    );
    assert_eq!(state(&harness), "off");
}
