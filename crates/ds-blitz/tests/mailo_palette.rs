//! The command panel, on a real Blitz document: a row's trailing action
//! fires without running, closing or selecting its row.

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::{Appearance, CommandPalette, Ds, Grain, Icon, Material, RowAction, SpaceLook};
use ds_blitz::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 480,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// A flat ground, so a pixel differs only where something is painted over it.
fn flat() -> SpaceLook {
    SpaceLook {
        grain: Grain(0),
        ..SpaceLook::default()
    }
}

/// Three recent searches, each with a remove that logs its row.
fn rows(mut log: Signal<Vec<String>>) -> Vec<ds::PaletteGroup<u8>> {
    let entries = (1..=3u8)
        .map(|value| ds::PaletteRow {
            action: Some(RowAction {
                icon: Icon::X,
                label: "Remove from recent".to_string(),
                on_press: EventHandler::new(move |_| {
                    log.with_mut(|log| log.push(format!("remove:{value}")))
                }),
            }),
            ..ds::PaletteRow::new(value, format!("recent search {value}"))
        })
        .collect();
    vec![ds::PaletteGroup::list("Recent", entries)]
}

#[component]
fn Page() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, look: flat(),
            // The root fills the view: the palette's layer is laid out inside it.
            p { class: "log", style: "height:440px; margin:0", {log().join(",")} }
            CommandPalette::<u8> {
                    label: "Search and commands".to_string(),
                    placeholder: "Search".to_string(),
                    query: String::new(),
                    tokens: Vec::new(),
                    groups: rows(log),
                    empty: "Nothing".to_string(),
                    oninput: move |_| {},
                    onpick: move |value: u8| log.with_mut(|log| log.push(format!("pick:{value}"))),
                    onclose: move |()| log.with_mut(|log| log.push("close".to_string())),
                    on_select: move |index: usize| log.with_mut(|log| log.push(format!("select:{index}"))),
            }
        }
    }
}

#[allow(non_snake_case)]
fn Opaque() -> Element {
    rsx! { Page {} }
}

#[test]
fn a_trailing_action_fires_without_picking_or_selecting_its_row() {
    let mut harness = Harness::new(Opaque, VIEW);
    harness.advance(ms(600));
    assert_eq!(harness.text_of(".log").as_deref(), Some("select:0"));
    // The second row's remove: the pointer crosses the row to reach it.
    let remove = ".ds-row:nth-child(3) .ds-row-action .ds-button";
    let at = harness.centre(remove).expect("the second row's remove");
    harness.click(at);
    harness.advance(ms(100));
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some("select:0,remove:2"),
        "the remove fired; nothing was picked, closed or selected"
    );
    assert_eq!(harness.count(".ds-palette"), 1, "the palette is still open");
}
