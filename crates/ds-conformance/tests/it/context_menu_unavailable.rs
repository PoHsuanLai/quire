//! A context menu leaves out the items that cannot be picked by default, keeps them dimmed under
//! `ContextUnavailable::Dim`, and a row's tip shows on hover even while the row is disabled.

use dioxus::prelude::*;
use ds::host::measure::Anchor;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::cell::Cell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 320,
    scale_percent: 100,
};

thread_local! {
    static MODE: Cell<ContextUnavailable> = const { Cell::new(ContextUnavailable::Hide) };
}

fn items() -> Vec<MenuItem<u8>> {
    vec![
        MenuItem::new(0, "Copy"),
        MenuItem::new(1, "Stop")
            .with_availability(Availability::Disabled)
            .with_tip("Wait for the agent to finish"),
        MenuItem::new(2, "Paste"),
    ]
}

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:100vh" }
            Menu {
                placement: MenuPlacement::Context,
                anchor: Anchor::Point(Point { x: Px(100.0), y: Px(60.0) }),
                items: items(),
                unavailable: MODE.with(Cell::get),
                onpick: move |_: u8| {},
                onclose: move |_| {},
            }
        }
    }
}

fn start(mode: ContextUnavailable) -> Harness {
    MODE.with(|cell| cell.set(mode));
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(300));
    harness
}

#[test]
fn a_context_menu_hides_what_cannot_be_picked_unless_asked_to_dim_it() {
    assert_eq!(start(ContextUnavailable::Hide).count(".ds-menu-item"), 2);
    let dimmed = start(ContextUnavailable::Dim);
    assert_eq!(dimmed.count(".ds-menu-item"), 3);
    assert_eq!(
        dimmed.count(".ds-menu-item[aria-disabled=true]"),
        1,
        "the unavailable row is kept, marked disabled"
    );
}

#[test]
fn a_disabled_row_shows_its_tip_on_hover() {
    let mut harness = start(ContextUnavailable::Dim);
    assert_eq!(harness.count(".ds-tooltip"), 0);
    let at = harness
        .centre(".ds-menu-item[aria-disabled=true]")
        .expect("the disabled row");
    harness.send(Input::pointer_move(at));
    harness.advance(Duration::from_millis(1300));
    assert_eq!(harness.count(".ds-tooltip"), 1);
    assert_eq!(
        harness
            .attr(".ds-menu-item[aria-disabled=true]", "aria-description")
            .as_deref(),
        Some("Wait for the agent to finish")
    );
}
