//! `SkeletonRow` on a real Blitz document, on the virtual clock (design/30 section 2.9): an
//! avatar circle and one or two bars at a settings row's height, the title wider than the line
//! under it, static; hidden, it fades out and `on_hidden` runs once at its settle.

use dioxus::prelude::*;
use ds::prelude::*;
use ds::style::tokens::row_scale::ROW_SCALE;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 360,
    height: 200,
    scale_percent: 100,
};

static SHOWN: GlobalSignal<Shown> = Signal::global(|| Shown::Visible);
static HIDDEN: GlobalSignal<u32> = Signal::global(|| 0);

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "width:340px",
                div { id: "two",
                    SkeletonRow { shown: SHOWN(), on_hidden: move |()| *HIDDEN.write() += 1 }
                }
                div { id: "one",
                    SkeletonRow { lines: SkeletonLines::One }
                }
                div { id: "real",
                    Row { leading: RowLeading::Text("AB".to_owned()), title: "Alex", size: ds::components::lists::row::size::RowSize::Settings }
                }
            }
        }
    }
}

fn harness() -> Harness {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(300));
    harness
}

#[test]
fn it_draws_an_avatar_and_a_wide_and_a_short_bar_at_a_rows_height() {
    let harness = harness();
    assert_eq!(harness.count("#two .ds-skeleton[data-shape=circle]"), 1);
    assert_eq!(harness.count("#two .ds-skeleton[data-shape=line]"), 2);
    assert_eq!(harness.count("#one .ds-skeleton[data-shape=line]"), 1);
    let row = harness.rect("#two .ds-skeleton-row").expect("the row");
    assert!(
        (row.size.height.0 - f32::from(ROW_SCALE.settings_height.0)).abs() < 0.5,
        "the settings row's token height: {row:?}"
    );
    let lines = harness
        .rect("#two .ds-skeleton-row-lines > .ds-skeleton:first-child")
        .expect("the title bar");
    let detail = harness
        .rect("#two .ds-skeleton-row-lines > .ds-skeleton:last-child")
        .expect("the second bar");
    assert!(
        lines.size.width.0 > detail.size.width.0 + 20.0,
        "{lines:?} {detail:?}"
    );
    let circle = harness
        .rect("#two .ds-skeleton[data-shape=circle]")
        .expect("the avatar");
    assert!(
        (circle.size.width.0 - circle.size.height.0).abs() < 0.5,
        "round: {circle:?}"
    );
    assert!(
        (circle.size.width.0 - f32::from(ROW_SCALE.avatar.0)).abs() < 0.5,
        "the row avatar's token size: {circle:?}"
    );
    let column = harness
        .rect("#two .ds-skeleton-row-lines")
        .expect("the text column");
    assert!(
        (lines.size.width.0 / column.size.width.0 - 0.62).abs() < 0.01
            && (detail.size.width.0 / column.size.width.0 - 0.38).abs() < 0.01,
        "the preset's proportions, 62% and 38% of the column: {lines:?} {detail:?} {column:?}"
    );
    assert_eq!(
        harness
            .attr("#two .ds-skeleton-row", "aria-hidden")
            .as_deref(),
        Some("true"),
        "decorative"
    );
}

#[test]
fn hidden_it_fades_and_on_hidden_runs_once_after_it_settles() {
    let mut harness = harness();
    harness.within(|| *SHOWN.write() = Shown::Hidden);
    harness.advance(ms(1));
    assert_eq!(
        harness
            .attr("#two .ds-skeleton-row", "data-presence")
            .as_deref(),
        Some("leaving")
    );
    assert_eq!(harness.within(|| *HIDDEN.peek()), 0);
    harness.advance(settle(Anim::MenuOut, MotionLevel::Standard) + ms(20));
    assert_eq!(harness.count("#two .ds-skeleton-row"), 0);
    assert_eq!(harness.within(|| *HIDDEN.peek()), 1);
    assert_eq!(
        harness.count("#one .ds-skeleton-row"),
        1,
        "its sibling stays"
    );
}

#[test]
fn it_is_as_tall_as_the_row_it_stands_for_both_reading_the_row_tokens() {
    let harness = harness();
    let real = harness.rect("#real .ds-row").expect("the row");
    let skeleton = harness.rect("#two .ds-skeleton-row").expect("the skeleton");
    assert!((real.size.height.0 - skeleton.size.height.0).abs() < 0.5);
    let mark = harness.rect("#real .ds-row-leading").expect("the leading");
    let circle = harness
        .rect("#two .ds-skeleton[data-shape=circle]")
        .expect("the avatar");
    assert!((mark.size.width.0 - circle.size.width.0).abs() < 0.5);
    assert!((mark.size.height.0 - f32::from(ROW_SCALE.avatar.0)).abs() < 0.5);
    let css = ds::stylesheet();
    assert!(css.contains("--row-settings-h:44px") && css.contains("--row-avatar:34px"));
    for sheet in [
        include_str!("../../ds/src/components/lists/row/row.css"),
        include_str!("../../ds/src/components/overlays/skeleton_row.css"),
    ] {
        assert!(
            !sheet.contains("44px") && !sheet.contains("34px"),
            "neither sheet restates the settings row's size: both read the row tokens"
        );
    }
}
