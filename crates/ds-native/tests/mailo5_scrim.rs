//! mailo gaps 5, a scrim drawn inline, on a real Blitz document: `Scrim { flow: Flow::Inline }`
//! stands in the pane that renders it (not in the overlay), covers exactly that pane's
//! positioned box, lies under the reader the pane draws after it (a press on the reader is the
//! reader's), and a press on the veil itself dismisses.

use dioxus::prelude::*;
use ds::{Appearance, Ds, Flow, Material, Point, Px, Scrim};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 420,
    scale_percent: 100,
};

/// A pane with a list, the scrim, and a peeked reader over the scrim's right half.
#[allow(non_snake_case)]
fn Page() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut shown = use_signal(|| true);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "padding:40px",
                div {
                    class: "pane",
                    style: "position:relative; width:400px; height:240px",
                    p { "The list." }
                    if shown() {
                        Scrim {
                            label: "Close peek",
                            flow: Flow::Inline,
                            onclose: move |_| {
                                shown.set(false);
                                log.with_mut(|log| log.push("dismiss".to_string()));
                            },
                        }
                        article {
                            class: "reader",
                            style: "position:absolute; left:200px; top:0; width:200px; height:240px",
                            onclick: move |_| log.with_mut(|log| log.push("reader".to_string())),
                            "The peeked reader."
                        }
                    }
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

#[test]
fn an_inline_scrim_sits_in_its_pane_under_the_reader_and_a_press_dismisses() {
    let mut harness = Harness::new(Page, VIEW);
    harness.advance(Duration::from_millis(50));
    assert_eq!(harness.count(".pane > .ds-scrim[*|data-flow=inline]"), 1);
    assert_eq!(
        harness.count(".ds-overlay .ds-scrim"),
        0,
        "no scrim in the overlay"
    );
    let pane = harness.rect(".pane").expect("the pane is laid out");
    let scrim = harness
        .rect(".pane > .ds-scrim")
        .expect("the scrim is laid out");
    assert_eq!(scrim, pane, "the scrim covers its pane exactly");

    // The reader was drawn after the scrim, so a press on it is the reader's.
    let reader = harness.centre(".reader").expect("the reader is laid out");
    harness.click(reader);
    harness.advance(Duration::from_millis(50));
    assert_eq!(harness.text_of(".log").as_deref(), Some("reader"));
    assert_eq!(harness.count(".pane > .ds-scrim"), 1);

    // The veil's uncovered left half dismisses.
    harness.click(Point {
        x: pane.origin.x + Px(60.0),
        y: pane.origin.y + Px(120.0),
    });
    harness.advance(Duration::from_millis(50));
    assert_eq!(harness.text_of(".log").as_deref(), Some("reader,dismiss"));
    assert_eq!(harness.count(".ds-scrim"), 0);
}
