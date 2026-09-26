//! The compact MonthGrid on a real Blitz document (design/04-COMPONENTS.md section 39; sill
//! Q190): at `MonthDensity::Auto` inside a small desktop `WidgetFrame`, a six-week month with
//! its step buttons lies wholly inside the frame's content box (164 less 12 padding a side,
//! 140 x 140, the widgets' measured inset, design/23 section 1.1), where the regular grid (224 x 254) did not; today's disc is round
//! and roomy enough for two digits (Q361). A layout, not a timing, so one look.

#[path = "../../ds/tests/support/month_sample.rs"]
mod month_sample;

use dioxus::prelude::*;
use ds::{
    Appearance, Ds, Material, MonthGrid, Rect, RootChrome, Step, WeekNumbers, WidgetFrame,
    WidgetMetrics, WidgetSize,
};
use ds_native::{Harness, Viewport};
use month_sample::{AUGUST, First, SEPTEMBER, sample};
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
        (140.0, 140.0),
        "the small frame's content box is 164 less 12 a side"
    );
    assert!(
        (body.origin.x.0 - card.origin.x.0 - 12.0).abs() < 0.01
            && (body.origin.y.0 - card.origin.y.0 - 12.0).abs() < 0.01,
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
        (140.0, 138.0),
        "the measured compact size"
    );
    // The last week's discs overhang their row by a pixel, still inside the frame.
    let foot = rect(&harness, ".ds-month-weeks > :last-child .ds-month-num");
    assert!(
        within(foot, body),
        "the last week's disc {foot:?} inside {body:?}"
    );
}

/// The same widget on September, whose today (the 26th) is two digits.
#[allow(non_snake_case)]
fn SmallSeptember() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Widget, chrome: Some(RootChrome::Transparent),
            div { style: WidgetMetrics::default().style_attr(),
                WidgetFrame { size: WidgetSize::Small,
                    MonthGrid { data: sample(SEPTEMBER, First::Monday), onstep: move |_: Step| {} }
                }
            }
        }
    }
}

/// Q361: the 16 px disc left two tabular digits (about 12 px at 700) touching its rim. Today's
/// disc is a circle twice the number's size (`--fs-caption` 10), so two digits keep about 4 px
/// a side, as the regular grid's 24 px disc does round its 11.5 px number.
#[test]
fn todays_compact_disc_is_a_circle_twice_its_number() {
    let mut harness = Harness::new(SmallSeptember, VIEW);
    harness.advance(Duration::from_millis(50));
    assert_eq!(
        harness
            .text_of(".ds-month-day[*|aria-current=date] .ds-month-num")
            .as_deref(),
        Some("26"),
        "today is two digits"
    );
    let disc = rect(&harness, ".ds-month-day[*|aria-current=date] .ds-month-num");
    assert_eq!(
        (disc.size.width.0, disc.size.height.0),
        (20.0, 20.0),
        "twice --fs-caption, round"
    );
    let cell = rect(&harness, ".ds-month-day[*|aria-current=date]");
    assert!(
        disc.origin.x.0 >= cell.origin.x.0 - 0.01
            && disc.origin.x.0 + disc.size.width.0 <= cell.origin.x.0 + cell.size.width.0 + 0.01,
        "the disc {disc:?} stays inside its column {cell:?}"
    );
}
