//! A flush `Row` (`line_height`) on a real Blitz document: the title and the detail are each
//! exactly the asked height, stacked, and start at the row's left edge; a row without it is
//! unchanged.

use dioxus::prelude::*;
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 240,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn FlushApp() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Popover,
            div { class: "flush", style: "position:absolute;left:20px;top:10px;width:300px",
                Row { title: "Title", detail: TextLine::from("Detail"), line_height: 17.0 }
            }
            div { class: "plain", style: "position:absolute;left:20px;top:100px;width:300px",
                Row { title: "Title", detail: TextLine::from("Detail") }
            }
        }
    }
}

fn start() -> Harness {
    let mut harness = Harness::new(
        FlushApp,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    harness.advance(Duration::from_millis(50));
    harness
}

fn rect(harness: &Harness, selector: &str) -> (f32, f32, f32, f32) {
    let rect = harness
        .rect(selector)
        .unwrap_or_else(|| panic!("{selector} is not in the document:\n{}", harness.html()));
    (
        rect.origin.x.0,
        rect.origin.y.0,
        rect.size.width.0,
        rect.size.height.0,
    )
}

#[test]
fn a_flush_rows_lines_are_exactly_the_asked_height_and_stacked_at_the_left_edge() {
    let harness = start();
    let row = rect(&harness, ".flush .ds-row");
    let title = rect(&harness, ".flush .ds-row-title");
    let detail = rect(&harness, ".flush .ds-row-detail");
    assert_eq!(title.3, 17.0, "the title line");
    assert_eq!(detail.3, 17.0, "the detail line");
    assert_eq!(title.0, row.0, "the title starts at the row's left edge");
    assert_eq!(detail.0, row.0, "the detail starts at the row's left edge");
    assert_eq!(title.1, row.1, "no leading space above the title");
    assert_eq!(detail.1, title.1 + 17.0, "the detail sits under the title");
    assert_eq!(row.3, 34.0, "no trailing space: the row is its two lines");
    assert_eq!(row.0, 20.0);
}

#[test]
fn a_row_without_line_height_keeps_its_own_height() {
    let harness = start();
    assert_eq!(
        harness.attr(".plain .ds-row", "data-density").as_deref(),
        Some("compact")
    );
    assert_eq!(
        harness.attr(".flush .ds-row", "data-density").as_deref(),
        Some("lines")
    );
    assert!(
        harness.html().contains("--row-line:17px"),
        "the caller's px is an inline custom property"
    );
    assert!(rect(&harness, ".plain .ds-row").3 >= 24.0);
}
