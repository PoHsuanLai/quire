//! `SkeletonRow` on a real Blitz document, on the virtual clock (design/30 section 2.9): an
//! avatar circle and one or two bars at a settings row's height, the title wider than the line
//! under it, static; hidden, it fades out and `on_hidden` runs once at its settle.

use dioxus::prelude::*;
use ds::prelude::*;
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
        row.size.height.0 >= 43.5,
        "a settings row's height: {row:?}"
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
