//! A sheet whose content fits: a touchpad drag stretches it only while the fingers hold it. The
//! stretch carries the content out from under the pointer; the lift still reaches the scroller
//! the fingers latched, and it springs back.

use dioxus::prelude::*;
use ds::host::gesture::GesturePhase;
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 1000,
    height: 700,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            Sheet { label: "Sheet", onclose: |_| {},
                div { class: "short", style: "height:200px", "content" }
            }
        }
    }
}

fn offset(harness: &Harness) -> f64 {
    harness.with_doc(|doc| {
        let id = doc
            .query_selector(".ds-sheet")
            .ok()
            .flatten()
            .expect("the sheet");
        doc.get_node(id).expect("the sheet").scroll_offset().y
    })
}

#[test]
fn a_dragged_sheet_whose_content_fits_springs_back_when_the_fingers_lift() {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(800));
    let short = harness.rect(".short").expect("the content");
    let at = Point {
        x: Px(short.origin.x.0 + 50.0),
        y: Px(short.origin.y.0 + 50.0),
    };
    harness.send(Input::fingers(at, Px(0.0), Px(0.0), GesturePhase::Began));
    for _ in 0..6 {
        harness.send(Input::fingers(at, Px(0.0), Px(40.0), GesturePhase::Changed));
        harness.advance(Duration::from_millis(16));
    }
    assert!(
        offset(&harness) < -20.0,
        "the drag stretches it: {}",
        offset(&harness)
    );
    harness.send(Input::fingers(at, Px(0.0), Px(0.0), GesturePhase::Ended));
    harness.advance(Duration::from_millis(1500));
    assert_eq!(offset(&harness), 0.0);
}
