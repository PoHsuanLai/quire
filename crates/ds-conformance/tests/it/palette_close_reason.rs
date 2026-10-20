//! A command palette tells its consumer why it asked to close: Escape, a pointer down outside
//! the card and a pick that closes are three different `PaletteClose` values.

use dioxus::prelude::*;
use ds::components::menus::palette::palette_group::{PaletteGroup, PaletteRow};
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 480,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[component]
fn Page() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            p { class: "log", style: "height:440px; margin:0", {log().join(",")} }
            CommandPalette::<u8> {
                label: "Commands".to_string(),
                placeholder: "Search".to_string(),
                query: String::new(),
                tokens: Vec::new(),
                groups: vec![PaletteGroup::list("Commands", vec![PaletteRow::new(1u8, "Run")])],
                empty: "Nothing".to_string(),
                oninput: move |_| {},
                onpick: move |_| {},
                onclose: move |why: PaletteClose| log.with_mut(|log| log.push(format!("{why:?}"))),
            }
        }
    }
}

#[allow(non_snake_case)]
fn Opaque() -> Element {
    rsx! { Page {} }
}

fn start() -> Harness {
    let mut harness = Harness::new(Opaque, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(600));
    harness
}

fn reported(harness: &Harness) -> Option<String> {
    harness.text_of(".log")
}

#[test]
fn escape_reports_escape() {
    let mut harness = start();
    harness.send(Input::key(ShortcutKey::Escape));
    harness.advance(ms(50));
    assert_eq!(reported(&harness).as_deref(), Some("Escape"));
}

#[test]
fn a_pointer_down_outside_the_card_reports_outside() {
    let mut harness = start();
    let corner = Point {
        x: Px(20.0),
        y: Px(20.0),
    };
    harness.send(Input::pointer_down(corner));
    harness.send(Input::pointer_up(corner));
    harness.advance(ms(50));
    assert_eq!(reported(&harness).as_deref(), Some("Outside"));
}

#[test]
fn a_pick_that_closes_reports_pick() {
    let mut harness = start();
    let run = harness
        .centre(".ds-row:nth-child(2) .ds-row-title")
        .expect("Run");
    harness.send(Input::click(run));
    harness.advance(ms(50));
    assert_eq!(reported(&harness).as_deref(), Some("Pick"));
}
