//! `text-overflow: ellipsis` draws its marker on Blitz (fork e7f9f877, with 851fa7dd's review
//! fixes): a nowrap line that overflows a clipping block keeps the glyphs that fit and ends in
//! `…`, and a line that fits is left whole. Read from the real document's inline layout, where
//! Blitz records each line's cut; the text itself stays whole for selection and hit testing.

use dioxus::prelude::*;
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const CUT: &str = "white-space: nowrap; overflow: hidden; text-overflow: ellipsis; width: 120px;";
const LONG: &str = "Quarterly report for the board meeting, final revised version three";
const SHORT: &str = "Notes";

#[allow(non_snake_case)]
fn Lines() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { id: "long", style: CUT, "{LONG}" }
            div { id: "short", style: CUT, "{SHORT}" }
        }
    }
}

fn start() -> Harness {
    let viewport = Viewport {
        width: 400,
        height: 200,
        scale_percent: 100,
    };
    let mut harness = Harness::new(
        Lines,
        HarnessConfig::new(viewport).with_clock(Clock::Virtual),
    );
    harness.advance(Duration::from_millis(100));
    harness
}

/// Where Blitz cut `selector`'s first line, as (glyphs kept, the marker's x): `None` for a line
/// drawn whole.
fn cut(harness: &Harness, selector: &str) -> Option<(usize, f32)> {
    harness.with_doc(|doc| {
        let id = doc
            .query_selector(selector)
            .ok()
            .flatten()
            .unwrap_or_else(|| panic!("no {selector}"));
        let text = doc
            .get_node(id)
            .and_then(|node| node.element_data())
            .and_then(|data| data.inline_layout_data.as_ref())
            .unwrap_or_else(|| panic!("{selector} lays out no text"));
        let marker = text.text_overflow.as_ref()?;
        let line = (*marker.lines.first()?)?;
        Some((line.glyphs_kept, line.marker_x))
    })
}

#[test]
fn a_line_too_long_for_its_clipping_box_ends_in_an_ellipsis_and_one_that_fits_is_whole() {
    let harness = start();
    let (kept, marker_x) = cut(&harness, "#long").expect("the long line is cut");
    let width = harness.rect("#long").expect("the long box").size.width.0;
    assert!(
        kept > 0 && kept < LONG.chars().count(),
        "{kept} glyphs of {} kept",
        LONG.chars().count()
    );
    assert!(
        marker_x > 0.0 && marker_x < width,
        "the ellipsis starts at {marker_x}, inside the {width} px box"
    );
    assert_eq!(
        cut(&harness, "#short"),
        None,
        "a line that fits is drawn whole"
    );
    assert_eq!(
        harness.text_of("#long").as_deref(),
        Some(LONG),
        "the text stays whole"
    );
}
