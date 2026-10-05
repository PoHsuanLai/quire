//! An image-only SegmentedControl segment has an accessible name (design/30 section 1.4,
//! `Choice::with_name`): the segment's `aria-label`, where a labelled one is named by its words.

use dioxus::prelude::*;
use ds::components::controls::segmented::Tracking;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 240,
    height: 80,
    scale_percent: 100,
};

#[path = "support/probe.rs"]
mod probe;

fn page(theme: Theme) -> Element {
    let icons = vec![
        Choice::new(0u8, "")
            .with_icon(IconSource::Glyph(Icon::Columns))
            .with_name("Columns"),
        Choice::new(1, "")
            .with_icon(IconSource::Glyph(Icon::Grid))
            .with_name("Grid"),
    ];
    rsx! {
        Ds { appearance: Appearance { theme, ..Appearance::default() }, material: Material::Window,
            div { class: "icons",
                SegmentedControl::<u8> { label: "Layout", choices: icons, tracking: Tracking::SelectOne(1), size: ControlSize::Regular, onchange: |_| {} }
            }
            div { class: "words",
                SegmentedControl::<u8> { label: "View", choices: Choice::pairs(vec![(0u8, "Day".to_owned()), (1, "Week".to_owned())]), tracking: Tracking::SelectOne(0), size: ControlSize::Regular, onchange: |_| {} }
            }
        }
    }
}

#[allow(non_snake_case)]
fn Light() -> Element {
    page(Theme::Light)
}

#[allow(non_snake_case)]
fn Dark() -> Element {
    page(Theme::Dark)
}

#[test]
fn an_image_only_segment_is_named_and_a_worded_one_is_left_to_its_words() {
    let mut harness = Harness::new(Light, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(50));
    probe::keep(&harness.render().expect("renders"), "segmented-name-light");
    let mut dark = Harness::new(Dark, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    dark.advance(Duration::from_millis(50));
    probe::keep(&dark.render().expect("renders"), "segmented-name-dark");
    let name = |selector: &str| harness.attr(selector, "aria-label");
    assert_eq!(
        name(".icons .ds-segmented-segment:nth-of-type(1)").as_deref(),
        Some("Columns")
    );
    assert_eq!(
        name(".icons .ds-segmented-segment:nth-of-type(2)").as_deref(),
        Some("Grid")
    );
    assert_eq!(name(".words .ds-segmented-segment:nth-of-type(1)"), None);
    assert_eq!(
        harness
            .text_of(".words .ds-segmented-segment:nth-of-type(1)")
            .as_deref(),
        Some("Day")
    );
}
