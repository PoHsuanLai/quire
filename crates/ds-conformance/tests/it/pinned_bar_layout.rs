//! PinnedBar on a real Blitz document: the bar lies over the container's top or bottom edge, and
//! the scroller beneath it keeps the whole container.

use dioxus::prelude::*;
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Query, Viewport};
use std::cell::Cell;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 300,
    scale_percent: 100,
};

thread_local! {
    static EDGE: Cell<PinEdge> = const { Cell::new(PinEdge::Top) };
}

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { class: "frame", style: "width:300px;height:200px;display:flex",
                PinnedBar {
                    label: "Suggestion",
                    edge: EDGE.with(Cell::get),
                    bar: rsx! { span { "Offer" } },
                    div { class: "scroller", style: "flex:1 1 auto;overflow:auto;min-height:0",
                        div { style: "height:600px" }
                    }
                }
            }
        }
    }
}

fn start(edge: PinEdge) -> Harness {
    EDGE.with(|cell| cell.set(edge));
    Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

#[test]
fn the_bar_lies_over_the_edge_it_is_pinned_to_and_the_scroller_keeps_the_whole_box() {
    let top = start(PinEdge::Top);
    let frame = top.rect(".frame").expect("frame");
    let bar = top.rect(".ds-pinned-bar").expect("bar");
    assert!(
        bar.origin.y.0 < frame.origin.y.0 + 20.0,
        "pinned at the top"
    );
    let scroller = top.rect(".scroller").expect("scroller");
    assert_eq!(scroller.size.height.0, frame.size.height.0);
    let bottom = start(PinEdge::Bottom);
    let frame = bottom.rect(".frame").expect("frame");
    let bar = bottom.rect(".ds-pinned-bar").expect("bar");
    assert!(
        bar.origin.y.0 + bar.size.height.0 > frame.origin.y.0 + frame.size.height.0 - 20.0,
        "pinned at the bottom"
    );
}
