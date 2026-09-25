//! AppearancePicker at the control center's width (sill FINDINGS Q101): in a 296 px
//! `ModulePanel` (a 320 px panel's grid, padding 12) the Full Motion row runs past the panel's
//! content box and clips "Reduced"; `PickerLayout::Compact` keeps every row inside it, and no
//! segment is narrower than its words. Measured on the laid-out rects.

use dioxus::prelude::*;
use ds::{
    Appearance, AppearancePicker, Ds, Material, ModuleGrid, ModulePanel, PickerLayout, Rect,
    SystemPrefs,
};
use ds_native::{Harness, Viewport};
use std::cell::Cell;
use std::time::Duration;

thread_local! {
    static LAYOUT: Cell<PickerLayout> = const { Cell::new(PickerLayout::Full) };
}

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
    scale_percent: 100,
};

/// The control center's panel: 320 wide, the grid's padding 12, so the module is 296.
#[allow(non_snake_case)]
fn Panel() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Popover,
            div { style: "width:320px",
                ModuleGrid {
                    ModulePanel {
                        AppearancePicker {
                            value: Appearance::default(),
                            system: SystemPrefs::default(),
                            onchange: |_| {},
                            layout: LAYOUT.get(),
                        }
                    }
                }
            }
        }
    }
}

fn laid_out(layout: PickerLayout) -> Harness {
    LAYOUT.set(layout);
    let mut harness = Harness::new(Panel, VIEW);
    harness.advance(Duration::from_millis(50));
    harness
}

fn rect(harness: &Harness, selector: &str) -> Rect {
    harness
        .rect(selector)
        .unwrap_or_else(|| panic!("{selector} is not in the document:\n{}", harness.html()))
}

fn right(rect: Rect) -> f32 {
    rect.origin.x.0 + rect.size.width.0
}

const ROWS: [&str; 3] = [
    ".ds-appearance-row:nth-child(1) .ds-segmented",
    ".ds-appearance-row:nth-child(2) .ds-appearance-swatches",
    ".ds-appearance-row:nth-child(3) .ds-segmented",
];

#[test]
fn the_module_is_296_and_the_full_motion_row_overflows_it() {
    let harness = laid_out(PickerLayout::Full);
    assert_eq!(rect(&harness, ".ds-module-panel").size.width.0, 296.0);
    let content = right(rect(&harness, ".ds-appearance"));
    let motion = right(rect(&harness, ROWS[2]));
    assert!(
        motion > content,
        "the Full Motion row ends at {motion}, inside the content's {content}: nothing to fix"
    );
}

#[test]
fn every_compact_row_fits_the_module_and_no_segment_is_squeezed() {
    let harness = laid_out(PickerLayout::Compact);
    let content = rect(&harness, ".ds-appearance");
    // The panel's content box: 296 less the padding 12 and the hairline each side.
    assert!(content.size.width.0 <= 272.0, "{content:?}");
    for row in ROWS {
        let end = right(rect(&harness, row));
        eprintln!("{row}: ends at {end}, content ends at {}", right(content));
        assert!(
            end <= right(content) + 0.5,
            "{row} ends at {end}, past the content's {}",
            right(content)
        );
    }
    // A segment is never narrower than its words: min-width stays auto, so a row that could
    // not fit would run past the content box above rather than clip a label.
    let last = rect(
        &harness,
        ".ds-appearance-row:nth-child(3) .ds-segment:last-child",
    );
    assert_eq!(
        harness
            .text_of(".ds-appearance-row:nth-child(3) .ds-segment:last-child")
            .as_deref(),
        Some("Reduced")
    );
    assert!(right(last) <= right(content) + 0.5, "Reduced: {last:?}");
}
