//! A command palette row that keeps the palette open (design/30 section 2.4): picking it yields
//! its value and leaves the palette up; a row that does not still closes it first.

use dioxus::prelude::*;
use ds::components::menus::palette::palette_group::{PaletteGroup, PaletteRow};
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
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
    let rows = vec![
        PaletteRow {
            after: AfterPick::KeepOpen,
            ..PaletteRow::new(1u8, "Toggle")
        },
        PaletteRow::new(2u8, "Run"),
    ];
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            p { class: "log", style: "height:440px; margin:0", {log().join(",")} }
            CommandPalette::<u8> {
                label: "Commands".to_string(),
                placeholder: "Search".to_string(),
                query: String::new(),
                tokens: Vec::new(),
                groups: vec![PaletteGroup::list("Commands", rows)],
                empty: "Nothing".to_string(),
                oninput: move |_| {},
                onpick: move |value: u8| log.with_mut(|log| log.push(format!("pick:{value}"))),
                onclose: move |()| log.with_mut(|log| log.push("close".to_string())),
            }
        }
    }
}

#[allow(non_snake_case)]
fn Opaque() -> Element {
    rsx! { Page {} }
}

#[test]
fn a_keep_open_row_yields_without_closing_and_an_ordinary_row_closes_first() {
    let mut harness = Harness::new(Opaque, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(600));
    let toggle = harness
        .centre(".ds-row:nth-child(2) .ds-row-title")
        .expect("Toggle");
    harness.send(Input::click(toggle));
    harness.advance(ms(50));
    assert_eq!(harness.text_of(".log").as_deref(), Some("pick:1"));
    assert_eq!(harness.count(".ds-palette"), 1, "still up");
    let run = harness
        .centre(".ds-row:nth-child(3) .ds-row-title")
        .expect("Run");
    harness.send(Input::click(run));
    harness.advance(ms(50));
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some("pick:1,close,pick:2")
    );
}
