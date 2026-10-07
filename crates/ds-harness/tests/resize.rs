//! `Harness::resize_window`: the document lays out again at the new size, as when a window edge is
//! dragged. The body keeps its 8 px margin, so a full-width box is 16 px short of the window.

use dioxus::prelude::*;
use ds_blitz::Extent;
use ds_harness::{Harness, Query, Viewport};

const NARROW: Viewport = Viewport {
    width: 320,
    height: 240,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Fill() -> Element {
    rsx! { div { id: "fill", style: "width:100%;height:100px;" } }
}

fn size_of(rect: ds_core::geometry::units::Rect) -> (f32, f32) {
    (rect.size.width.0, rect.size.height.0)
}

#[test]
fn a_resized_window_lays_the_document_out_at_the_new_size() {
    let mut harness = Harness::new(Fill, NARROW);
    let before = harness.rect("#fill").expect("the fill");
    assert_eq!(size_of(before), (304.0, 100.0));

    harness.resize_window(Extent::new(640, 480));
    let wide = harness.rect("#fill").expect("the fill");
    assert_eq!(size_of(wide), (624.0, 100.0));
    assert_eq!(harness.window_size(), Extent::new(640, 480));

    harness.resize_window(Extent::new(320, 240));
    let back = harness.rect("#fill").expect("the fill");
    assert_eq!(size_of(back), (304.0, 100.0));
}
