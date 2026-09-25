//! Sheet and modal parts, Q94: a root holding only a Sheet is 0 px tall unless it asks for the
//! viewport. `RootExtent::Viewport` makes it the viewport's size, measured on a real Blitz
//! document, and the sheet it hosts has that area to be placed in.

use dioxus::prelude::*;
use ds::{Appearance, Ds, Material, RootExtent, Sheet};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 400,
    scale_percent: 100,
};

/// An overlay root, `extent` as asked, holding one small sheet.
#[component]
fn Overlay(extent: RootExtent) -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet, extent,
            Sheet { label: "Power", onclose: move |_| {},
                p { "Shut down?" }
            }
        }
    }
}

#[allow(non_snake_case)]
fn Content() -> Element {
    rsx! { Overlay { extent: RootExtent::Content } }
}

#[allow(non_snake_case)]
fn Full() -> Element {
    rsx! { Overlay { extent: RootExtent::Viewport } }
}

fn laid_out(app: fn() -> Element) -> Harness {
    let mut harness = Harness::new(app, VIEW);
    harness.advance(Duration::from_millis(600));
    harness
}

#[test]
fn a_content_root_of_only_a_sheet_is_zero_tall() {
    let harness = laid_out(Content);
    let root = harness.rect(".ds").expect("the root is laid out");
    assert_eq!(root.size.height.0, 0.0, "the trap Q94 names: {root:?}");
    // The overlay fills the 0 px root, so the sheet's `max-height:calc(100% - 72px)` is
    // nothing: the sheet draws no body at all.
    let sheet = harness.rect(".ds-sheet").expect("the sheet is laid out");
    assert_eq!(sheet.size.height.0, 0.0, "{sheet:?}");
}

#[test]
fn a_viewport_root_is_the_viewports_size_and_its_sheet_is_placed_in_it() {
    let harness = laid_out(Full);
    assert_eq!(
        harness.attr(".ds", "data-extent").as_deref(),
        Some("viewport")
    );
    let root = harness.rect(".ds").expect("the root is laid out");
    assert_eq!(root.size.height.0, VIEW.height as f32, "{root:?}");
    assert_eq!(root.size.width.0, VIEW.width as f32, "{root:?}");
    let sheet = harness.rect(".ds-sheet").expect("the sheet is laid out");
    assert!(
        sheet.size.height.0 > 20.0,
        "the sheet has its content's height: {sheet:?}"
    );
    assert!(
        sheet.origin.y.0 + sheet.size.height.0 <= VIEW.height as f32,
        "the sheet is inside the viewport: {sheet:?}"
    );
}
