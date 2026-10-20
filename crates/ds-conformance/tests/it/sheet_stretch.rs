//! A long sheet under a touchpad drag past its top stretches only while the fingers hold it. The
//! stretch carries the content out from under the pointer; the lift still reaches the scroller the
//! fingers latched, and it springs back.

use dioxus::prelude::*;
use ds::host::gesture::GesturePhase;
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};
use std::cell::Cell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 1000,
    height: 700,
    scale_percent: 100,
};

thread_local! {
    static CONTENT: Cell<u32> = const { Cell::new(200) };
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let height = CONTENT.with(Cell::get);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            Sheet { label: "Sheet", onclose: |_| {},
                div { class: "content", style: "flex:none;height:{height}px", "content" }
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

/// A sheet with `height` px of content, its fingers down and pulled 240 px down from its top;
/// the offset while they hold it.
fn pulled(height: u32) -> (Harness, Point, f64) {
    CONTENT.with(|cell| cell.set(height));
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(800));
    let content = harness.rect(".content").expect("the content");
    let at = Point {
        x: Px(content.origin.x.0 + 50.0),
        y: Px(content.origin.y.0 + 50.0),
    };
    harness.send(Input::fingers(at, Px(0.0), Px(0.0), GesturePhase::Began));
    for _ in 0..6 {
        harness.send(Input::fingers(at, Px(0.0), Px(40.0), GesturePhase::Changed));
        harness.advance(Duration::from_millis(16));
    }
    let held = offset(&harness);
    (harness, at, held)
}

#[test]
fn a_long_sheet_dragged_past_its_top_springs_back_when_the_fingers_lift() {
    let (mut harness, at, held) = pulled(2000);
    assert!(held < -20.0, "the drag stretches it: {held}");
    harness.send(Input::fingers(at, Px(0.0), Px(0.0), GesturePhase::Ended));
    harness.advance(Duration::from_millis(1500));
    assert_eq!(offset(&harness), 0.0);
}
