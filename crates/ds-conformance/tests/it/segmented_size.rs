//! SegmentedControl on the size ladder (design/29-SIZING.md decision 5): the well is a control's
//! height (Mini 20, Small 24, Regular 28, Large 32, ExtraLarge 40), each segment fills it less the 1 px knob inset
//! (18, 22, 26, 30, 38), all whole pixels, at scale 1 and 2. Measured on the laid-out rects.

use dioxus::prelude::*;
use ds::components::controls::segmented::Tracking;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Query, Viewport};
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
            for (class , size) in [("mini", ControlSize::Mini), ("small", ControlSize::Small), ("regular", ControlSize::Regular), ("large", ControlSize::Large), ("xl", ControlSize::ExtraLarge)] {
                div { class,
                    SegmentedControl::<u8> {
                        label: "View",
                        choices: Choice::pairs(views.clone()),
                        tracking: Tracking::SelectOne(1),
                        size,
                        onchange: |_| {},
                    }
                }
            }
        }
    }
}

#[test]
fn the_well_and_its_segments_stand_on_the_ladder() {
    // (scope, well, segment), both from the ladder.
    let cases: Vec<(&str, f32, f32)> = [
        (".mini", ControlSize::Mini),
        (".small", ControlSize::Small),
        (".regular", ControlSize::Regular),
        (".large", ControlSize::Large),
        (".xl", ControlSize::ExtraLarge),
    ]
    .into_iter()
    .map(|(scope, size)| {
        let scale = size.scale();
        (
            scope,
            f32::from(scale.height.0),
            f32::from(scale.segment().0),
        )
    })
    .collect();
    for scale_percent in [100, 200] {
        let viewport = Viewport {
            width: 400,
            height: 240,
            scale_percent,
        };
        let mut harness = Harness::new(
            Both,
            HarnessConfig::new(viewport).with_clock(Clock::Virtual),
        );
        harness.advance(Duration::from_millis(50));
        for &(scope, well, segment) in &cases {
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
                height(".ds-segmented-segment:nth-child(1)"),
                segment,
                "{scope} at {scale_percent}"
            );
        }
    }
}
