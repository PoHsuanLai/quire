//! SegmentedControl on the size ladder (design/29-SIZING.md decision 5): the well is a control's
//! height (Regular 22, Small 16), each segment fills it less the 1 px knob inset (20, 14), both
//! whole pixels, at scale 1 and 2. Measured on the laid-out rects.

use dioxus::prelude::*;
use ds::{Appearance, Ds, Material, SegSize, SegmentedControl};
use ds_native::{Harness, Viewport};
use std::time::Duration;

#[allow(non_snake_case)]
fn Both() -> Element {
    let views = vec![
        (0u8, "Day".to_owned()),
        (1, "Week".to_owned()),
        (2, "Month".to_owned()),
    ];
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { class: "regular",
                SegmentedControl::<u8> { label: "View", options: views.clone(), value: 1, onchange: |_| {} }
            }
            div { class: "small",
                SegmentedControl::<u8> { label: "View", options: views, value: 1, size: SegSize::Small, onchange: |_| {} }
            }
        }
    }
}

#[test]
fn the_well_and_its_segments_stand_on_the_ladder() {
    // (scope, well, segment)
    const CASES: &[(&str, f32, f32)] = &[(".regular", 22.0, 20.0), (".small", 16.0, 14.0)];
    for scale_percent in [100, 200] {
        let viewport = Viewport {
            width: 400,
            height: 200,
            scale_percent,
        };
        let mut harness = Harness::new(Both, viewport);
        harness.advance(Duration::from_millis(50));
        for &(scope, well, segment) in CASES {
            let height = |selector: &str| {
                harness
                    .rect(&format!("{scope} {selector}"))
                    .unwrap_or_else(|| panic!("{scope} {selector} is laid out"))
                    .size
                    .height
                    .0
            };
            assert_eq!(height(".ds-segmented"), well, "{scope} at {scale_percent}");
            assert_eq!(
                height(".ds-segment:nth-child(1)"),
                segment,
                "{scope} at {scale_percent}"
            );
        }
    }
}
