//! SplitView's last pane keeps its least (design/30 section 2.7, `least_rest`): when the view is
//! too narrow for every pane's width, the pane before it gives up width down to its own least,
//! its body follows it, and the last pane is never drawn under its least. With room, nothing
//! changes; with no `least_rest`, the pane keeps its width as before.

use dioxus::prelude::*;
use ds::components::chrome::split_view::model::{PaneSpec, SplitPane};
use ds::components::chrome::split_view::view::SplitView;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Query, Viewport};
use std::time::Duration;

/// The least the content beside the sidebar is drawn at.
const LEAST_REST: f32 = 400.0;

#[allow(non_snake_case)]
fn Kept() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:180px",
                SplitView {
                    label: "Example",
                    panes: vec![SplitPane::new(PaneSpec::SIDEBAR, rsx! { p { "Side" } })],
                    least_rest: Px(LEAST_REST),
                    p { "Content" }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn Unkept() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:180px",
                SplitView {
                    label: "Example",
                    panes: vec![SplitPane::new(PaneSpec::SIDEBAR, rsx! { p { "Side" } })],
                    p { "Content" }
                }
            }
        }
    }
}

fn open(page: fn() -> Element, width: u32) -> Harness {
    let view = Viewport {
        width,
        height: 200,
        scale_percent: 100,
    };
    let mut harness = Harness::new(page, HarnessConfig::new(view).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(600));
    harness
}

fn width_of(harness: &Harness, selector: &str) -> f32 {
    harness
        .rect(selector)
        .map_or(-1.0, |rect| rect.size.width.0)
}

#[test]
fn a_narrow_view_squeezes_the_pane_to_its_least_before_the_rest() {
    // 560 px holds the sidebar's least (180) and the rest's least (400), not its preferred 240.
    let harness = open(Kept, 560);
    let pane = width_of(&harness, ".ds-split-pane");
    let body = width_of(&harness, ".ds-split-pane-body");
    let rest = width_of(&harness, ".ds-split-rest");
    assert!(
        rest >= LEAST_REST - 0.5,
        "the rest is {rest}px, under its least"
    );
    assert!(
        (180.0 - 0.5..240.0).contains(&pane),
        "the pane is {pane}px: not squeezed, or past its least"
    );
    assert!(
        (body - pane).abs() < 0.5,
        "the body is {body}px in a {pane}px pane: it did not follow the squeeze"
    );
}

#[test]
fn with_room_the_pane_keeps_its_width() {
    let harness = open(Kept, 900);
    assert_eq!(width_of(&harness, ".ds-split-pane"), 240.0);
    assert_eq!(width_of(&harness, ".ds-split-pane-body"), 240.0);
}

#[test]
fn without_a_least_rest_the_pane_keeps_its_width_however_narrow() {
    let harness = open(Unkept, 560);
    assert_eq!(width_of(&harness, ".ds-split-pane"), 240.0);
}
