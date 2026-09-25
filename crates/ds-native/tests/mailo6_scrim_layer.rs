//! mailo gaps 6, an inline scrim's own layer, on a real Blitz document: a pane draws the scrim
//! and then its positioned rows (`position:relative`, as a row with a hover strip is). With no
//! layer the rows paint over the scrim and a press on a row is the row's; with
//! `layer: ZLayer::Raise` the scrim is above them, a press on the same spot is the scrim's, and
//! it dismisses.

use dioxus::prelude::*;
use ds::{Appearance, Ds, Flow, Material, Point, Scrim, ZLayer};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 600,
    height: 360,
    scale_percent: 100,
};

/// The pane: a scrim in `layer`, then three positioned rows. Logs presses on rows and dismissals.
fn pane(layer: Option<ZLayer>) -> Element {
    let mut log = use_signal(Vec::<String>::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { class: "pane", style: "position:relative; width:400px; height:240px; margin:40px",
                Scrim {
                    label: "Close peek",
                    flow: Flow::Inline,
                    layer,
                    onclose: move |_| log.with_mut(|log| log.push("dismiss".to_string())),
                }
                for n in 1..=3 {
                    div {
                        class: "row row-{n}",
                        style: "position:relative; height:40px",
                        onclick: move |_| log.with_mut(|log| log.push(format!("row-{n}"))),
                        "Row {n}"
                    }
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

#[allow(non_snake_case)]
fn Bare() -> Element {
    pane(None)
}

#[allow(non_snake_case)]
fn Raised() -> Element {
    pane(Some(ZLayer::Raise))
}

/// The middle of the second row, and a harness settled on `app`.
fn settled(app: fn() -> Element) -> (Harness, Point) {
    let mut harness = Harness::new(app, VIEW);
    harness.advance(Duration::from_millis(50));
    let row = harness
        .centre(".row-2")
        .unwrap_or_else(|| panic!("the row is laid out:\n{}", harness.html()));
    (harness, row)
}

#[test]
fn without_a_layer_a_positioned_row_paints_over_the_inline_scrim() {
    let (mut harness, row) = settled(Bare);
    assert_eq!(harness.attr(".ds-scrim", "style"), None);
    assert!(harness.hits(row, ".row-2"), "the row is on top");
    assert!(!harness.hits(row, ".ds-scrim"));
    harness.click(row);
    harness.advance(Duration::from_millis(50));
    assert_eq!(harness.text_of(".log").as_deref(), Some("row-2"));
}

#[test]
fn an_inline_scrim_on_the_raise_layer_covers_the_positioned_rows() {
    let (mut harness, row) = settled(Raised);
    assert_eq!(
        harness.attr(".ds-scrim", "style").as_deref(),
        Some("z-index:var(--z-raise)")
    );
    assert!(harness.hits(row, ".ds-scrim"), "the scrim is on top");
    assert!(!harness.hits(row, ".row-2"));
    harness.click(row);
    harness.advance(Duration::from_millis(50));
    assert_eq!(harness.text_of(".log").as_deref(), Some("dismiss"));
}
