//! Where Blitz paints an absolutely positioned layer placed before a positioned block
//! (FINDINGS "Edit surface 2"): mailo's selection layer before its positioned `.c-body` was
//! painted over the text. CSS 2.1 Appendix E paints positioned descendants with `z-index: auto`
//! in tree order (step 8), so the later block belongs on top of the earlier layer; a block that
//! is not positioned is painted in step 3/7, under every positioned box.

use dioxus::prelude::*;
use ds::{EditSurface, ExtraClass};
use ds_native::{Harness, Viewport};

const VIEW: Viewport = Viewport {
    width: 200,
    height: 120,
    scale_percent: 100,
};

/// A red layer, then a blue block over the same box: the block positioned or not.
fn scene(block: &str) -> Element {
    rsx! {
        div { style: "position:relative; width:100px; height:100px; background:white",
            div { style: "position:absolute; left:0; top:0; width:100px; height:100px; background:rgb(255,0,0)" }
            div { style: "{block} width:100px; height:100px; background:rgb(0,0,255)" }
        }
    }
}

#[allow(non_snake_case)]
fn PositionedAfter() -> Element {
    scene("position:relative;")
}

#[allow(non_snake_case)]
fn StaticAfter() -> Element {
    scene("")
}

#[allow(non_snake_case)]
fn RaisedAfter() -> Element {
    scene("position:relative; z-index:1;")
}

#[allow(non_snake_case)]
fn LayerBelow() -> Element {
    rsx! {
        div { style: "position:relative; width:100px; height:100px; background:white",
            div { style: "position:absolute; left:0; top:0; width:100px; height:100px; background:rgb(255,0,0); z-index:-1" }
            div { style: "position:relative; width:100px; height:100px" , "" }
        }
    }
}

/// The colour at the middle of the box.
fn middle(app: fn() -> Element) -> [u8; 3] {
    let mut harness = Harness::new(app, VIEW);
    let picture = harness.render().expect("a picture");
    let [r, g, b, _] = picture.get_pixel(50, 50).0;
    [r, g, b]
}

const RED: [u8; 3] = [255, 0, 0];
const BLUE: [u8; 3] = [0, 0, 255];

#[test]
fn a_positioned_block_after_an_absolute_layer_paints_over_it() {
    assert_eq!(
        middle(PositionedAfter),
        BLUE,
        "CSS 2.1 E step 8: tree order"
    );
}

#[test]
fn a_static_block_paints_under_a_positioned_layer() {
    assert_eq!(
        middle(StaticAfter),
        RED,
        "CSS 2.1 E: step 3/7 before step 8"
    );
}

#[test]
fn a_raised_block_paints_over_the_layer() {
    assert_eq!(middle(RaisedAfter), BLUE);
}

#[test]
fn a_layer_at_z_index_minus_one_goes_under_its_containers_own_background() {
    // The container is positioned with `z-index: auto`, so it makes no stacking context: the
    // layer is painted in the root's step 2, before the container's white background (step 4).
    assert_eq!(
        middle(LayerBelow),
        [255, 255, 255],
        "CSS 2.1 E: negative layers first"
    );
}

/// mailo's shape: a translucent selection layer, then the edit surface itself made positioned
/// by the app's own class, its text over the layer.
#[allow(non_snake_case)]
fn SurfaceAfterLayer() -> Element {
    rsx! {
        style { ".c-body{{position:relative;color:rgb(0,0,0);font-size:60px;line-height:80px}}" }
        div { style: "position:relative; width:180px; height:100px; background:white",
            div { style: "position:absolute; left:0; top:0; width:180px; height:100px; background:rgb(255,0,0)" }
            EditSurface { extra_class: ExtraClass::parse("c-body").ok(), on_input: |_| {},
                p { "data-edit-node": "0", style: "margin:0", "WW" }
            }
        }
    }
}

/// The same with the surface not positioned: the layer covers the text.
#[allow(non_snake_case)]
fn StaticSurfaceAfterLayer() -> Element {
    rsx! {
        style { ".c-body{{color:rgb(0,0,0);font-size:60px;line-height:80px}}" }
        div { style: "position:relative; width:180px; height:100px; background:white",
            div { style: "position:absolute; left:0; top:0; width:180px; height:100px; background:rgb(255,0,0)" }
            EditSurface { extra_class: ExtraClass::parse("c-body").ok(), on_input: |_| {},
                p { "data-edit-node": "0", style: "margin:0", "WW" }
            }
        }
    }
}

/// How many pixels of the text's box are near black.
fn inked(app: fn() -> Element) -> usize {
    let mut harness = Harness::new(app, VIEW);
    let picture = harness.render().expect("a picture");
    (0..160u32)
        .flat_map(|x| (0..80u32).map(move |y| (x, y)))
        .filter(|&(x, y)| {
            let [r, g, b, _] = picture.get_pixel(x, y).0;
            r < 60 && g < 60 && b < 60
        })
        .count()
}

#[test]
fn a_positioned_surfaces_text_paints_over_a_layer_before_it() {
    assert!(inked(SurfaceAfterLayer) > 200, "the text is on top");
}

#[test]
fn a_static_surfaces_text_is_covered_by_a_positioned_layer_before_it() {
    assert_eq!(
        inked(StaticSurfaceAfterLayer),
        0,
        "the layer is on top, as CSS says"
    );
}
