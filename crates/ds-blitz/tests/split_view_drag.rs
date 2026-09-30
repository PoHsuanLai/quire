//! SplitView on a real Blitz document (design/30 section 2.7): dragging the divider sizes the
//! pane 1:1 within its widths, a drag past half its least folds it and reports it, and a
//! double-click returns it to its preferred width.

use dioxus::prelude::*;
use ds::{Appearance, Ds, Material, PaneSpec, Point, Px, Shown, SplitPane, SplitView};
use ds_harness::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 700,
    height: 200,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut shown = use_signal(|| Shown::Visible);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:180px",
                SplitView {
                    label: "Example",
                    panes: vec![SplitPane::new(PaneSpec::SIDEBAR, rsx! { p { "Side" } }).shown(shown())],
                    on_shown: move |(_, next)| shown.set(next),
                    p { "Content" }
                }
            }
            p { class: "shown", "{ds::Word::slug(shown())}" }
        }
    }
}

fn width(harness: &Harness) -> f32 {
    harness
        .rect(".ds-split-pane")
        .map_or(-1.0, |rect| rect.size.width.0)
}

fn harness() -> Harness {
    let mut harness =
        Harness::with_config(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(600));
    harness
}

fn divider(harness: &Harness) -> Point {
    harness.centre(".ds-split-divider").expect("divider")
}

#[test]
fn dragging_the_divider_sizes_the_pane_within_its_widths() {
    let mut harness = harness();
    assert_eq!(width(&harness), 240.0);
    let from = divider(&harness);
    harness.drag(
        from,
        Point {
            x: Px(from.x.0 + 40.0),
            y: from.y,
        },
        8,
    );
    harness.advance(Duration::from_millis(600));
    assert_eq!(width(&harness), 280.0);
    let from = divider(&harness);
    harness.drag(
        from,
        Point {
            x: Px(from.x.0 + 200.0),
            y: from.y,
        },
        8,
    );
    harness.advance(Duration::from_millis(600));
    assert_eq!(width(&harness), 320.0, "held to the most");
}

#[test]
fn a_drag_past_half_the_least_folds_the_pane_and_a_double_click_reopens_it() {
    let mut harness = harness();
    let from = divider(&harness);
    harness.drag(
        from,
        Point {
            x: Px(from.x.0 - 200.0),
            y: from.y,
        },
        10,
    );
    harness.advance(Duration::from_millis(1500));
    assert_eq!(harness.text_of(".shown").as_deref(), Some("hidden"));
    assert!(width(&harness) < 1.0, "folded: {}", width(&harness));
    let at = divider(&harness);
    harness.click(at);
    harness.click(at);
    harness.advance(Duration::from_millis(1500));
    assert_eq!(harness.text_of(".shown").as_deref(), Some("visible"));
    assert_eq!(width(&harness), 240.0);
}
