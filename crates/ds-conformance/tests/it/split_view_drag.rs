//! SplitView on a real Blitz document (design/30 section 2.7): dragging the divider sizes the
//! pane 1:1 within its widths, a drag past half its least folds it and reports it, and a
//! double-click returns it to its preferred width.

use dioxus::prelude::*;
use ds::components::chrome::split_view::model::{PaneSize, PaneSpec, SplitPane};
use ds::components::chrome::split_view::view::SplitView;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 700,
    height: 200,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut shown = use_signal(|| Shown::Visible);
    let mut saved = use_signal(Vec::<PaneSize>::new);
    let mut saves = use_signal(|| 0usize);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:180px",
                SplitView {
                    label: "Example",
                    panes: vec![SplitPane::new(PaneSpec::SIDEBAR, rsx! { p { "Side" } }).shown(shown())],
                    on_shown: move |(_, next)| shown.set(next),
                    on_resized: move |sizes| {
                        saved.set(sizes);
                        saves += 1;
                    },
                    p { "Content" }
                }
            }
            p { class: "shown", "{Word::slug(shown())}" }
            p { class: "saves", "{saves()}" }
            p { class: "saved", "{saved():?}" }
        }
    }
}

fn width(harness: &Harness) -> f32 {
    harness
        .rect(".ds-split-pane")
        .map_or(-1.0, |rect| rect.size.width.0)
}

fn harness() -> Harness {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
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
    harness.send(Input::drag(
        from,
        Point {
            x: Px(from.x.0 + 40.0),
            y: from.y,
        },
        8,
    ));
    harness.advance(Duration::from_millis(600));
    assert_eq!(width(&harness), 280.0);
    assert_eq!(
        harness.text_of(".saves").as_deref(),
        Some("1"),
        "one report for the whole drag, not one per frame"
    );
    assert_eq!(
        harness.text_of(".saved").as_deref(),
        Some("[Fixed(Px(280.0))]")
    );
    let from = divider(&harness);
    harness.send(Input::drag(
        from,
        Point {
            x: Px(from.x.0 + 200.0),
            y: from.y,
        },
        8,
    ));
    harness.advance(Duration::from_millis(600));
    assert_eq!(width(&harness), 320.0, "held to the most");
}

#[test]
fn a_drag_past_half_the_least_folds_the_pane_and_a_double_click_reopens_it() {
    let mut harness = harness();
    let from = divider(&harness);
    harness.send(Input::drag(
        from,
        Point {
            x: Px(from.x.0 - 200.0),
            y: from.y,
        },
        10,
    ));
    harness.advance(Duration::from_millis(1500));
    assert_eq!(harness.text_of(".shown").as_deref(), Some("hidden"));
    assert!(width(&harness) < 1.0, "folded: {}", width(&harness));
    let at = divider(&harness);
    harness.send(Input::click(at));
    harness.send(Input::click(at));
    harness.advance(Duration::from_millis(1500));
    assert_eq!(harness.text_of(".shown").as_deref(), Some("visible"));
    assert_eq!(width(&harness), 240.0);
}
