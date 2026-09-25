//! The compact MonthGrid on a real Blitz document (design/04-COMPONENTS.md section 39; sill
//! Q190): at `MonthDensity::Auto` inside a small desktop `WidgetFrame`, a six-week month with
//! its step buttons lies wholly inside the frame's content box (164 less 16 padding a side,
//! 132 x 132), where the regular grid (224 x 254) did not. A layout, not a timing, so one look.

#[path = "../../ds/tests/support/month_sample.rs"]
mod month_sample;

use dioxus::prelude::*;
use ds::{
    Appearance, Ds, Material, MonthGrid, Rect, RootChrome, Step, WeekNumbers, WidgetFrame,
    WidgetMetrics, WidgetSize,
};
use ds_native::{Harness, Viewport};
use month_sample::{AUGUST, First, sample};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 320,
    height: 320,
    scale_percent: 100,
};

/// A small desktop widget holding the month, as sill's calendar widget draws it.
#[allow(non_snake_case)]
fn SmallCalendar() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Widget, chrome: Some(RootChrome::Transparent),
            div { style: WidgetMetrics::default().style_attr(),
                WidgetFrame { size: WidgetSize::Small,
                    MonthGrid {
                        data: sample(AUGUST, First::Monday),
                        weeks: WeekNumbers::Show,
                        onstep: move |_: Step| {},
                    }
                }
            }
        }
    }
}

fn rect(harness: &Harness, selector: &str) -> Rect {
    harness
        .rect(selector)
        .unwrap_or_else(|| panic!("{selector} is missing:\n{}", harness.html()))
}

/// Whether `inner` lies within `outer`, to a hundredth of a pixel.
fn within(inner: Rect, outer: Rect) -> bool {
    let slack = 0.01;
    inner.origin.x.0 >= outer.origin.x.0 - slack
        && inner.origin.y.0 >= outer.origin.y.0 - slack
        && inner.origin.x.0 + inner.size.width.0 <= outer.origin.x.0 + outer.size.width.0 + slack
        && inner.origin.y.0 + inner.size.height.0 <= outer.origin.y.0 + outer.size.height.0 + slack
}

#[test]
fn the_compact_grid_fits_a_small_widget() {
    let mut harness = Harness::new(SmallCalendar, VIEW);
    harness.advance(Duration::from_millis(50));
    assert_eq!(
        harness.attr(".ds-month", "data-density").as_deref(),
        Some("compact")
    );
    let card = rect(&harness, ".ds-widget");
    let body = rect(&harness, ".ds-widget-body");
    let grid = rect(&harness, ".ds-month");
    let last = rect(&harness, ".ds-month-weeks");
    assert_eq!(
        (body.size.width.0, body.size.height.0),
        (132.0, 132.0),
        "the small frame's content box is 164 less 16 a side"
    );
    assert!(
        (body.origin.x.0 - card.origin.x.0 - 16.0).abs() < 0.01
            && (body.origin.y.0 - card.origin.y.0 - 16.0).abs() < 0.01,
        "the body is the card's content box: {card:?} {body:?}"
    );
    assert_eq!(
        harness.html().matches("class=\"ds-month-row\"").count(),
        1 + 6,
        "the heads and six weeks"
    );
    for part in [grid, last] {
        assert!(within(part, body), "{part:?} inside {body:?}");
    }
    let header = rect(&harness, ".ds-month-header");
    let heads = rect(&harness, ".ds-month-row[*|data-row=heads]");
    for (name, part) in [("header", header), ("heads", heads)] {
        assert!(within(part, body), "{name} {part:?} inside {body:?}");
    }
    for selector in [".ds-month-step", ".ds-month-day:last-child"] {
        let part = rect(&harness, selector);
        assert!(within(part, body), "{selector} {part:?} inside {body:?}");
    }
    // The drawn extent: the widest row by the header's top to the last week's foot.
    let tall = last.origin.y.0 + last.size.height.0 - header.origin.y.0;
    assert_eq!(
        (heads.size.width.0, tall),
        (126.0, 132.0),
        "the measured compact size"
    );
}
