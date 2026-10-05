//! PopUpButton's `start` and `anchor` on a real Blitz document: a pop-up that starts open draws
//! its menu once the button is laid out, against the button; with an `anchor` it draws it
//! there instead. Shots in light and dark.

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::components::menus::pop_up_button::{PopUpButton, PopUpKind};
use ds::host::measure::Anchor;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Query, Viewport};
use probe::{keep, rect};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 420,
    height: 260,
    scale_percent: 100,
};

fn page(theme: Theme, anchor: Option<Anchor>) -> Element {
    rsx! {
        Ds { appearance: Appearance { theme, ..Appearance::default() }, material: Material::Window, extent: RootExtent::Viewport,
            div { style: "padding:24px",
                PopUpButton::<u8> {
                    items: vec![MenuItem::new(1, "Rename"), MenuItem::new(2, "Delete")],
                    onpick: |_| {},
                    kind: PopUpKind::PullDown,
                    title: "Actions".to_string(),
                    start: Shown::Visible,
                    anchor,
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn Light() -> Element {
    page(Theme::Light, None)
}

#[allow(non_snake_case)]
fn Dark() -> Element {
    page(Theme::Dark, None)
}

#[allow(non_snake_case)]
fn Placed() -> Element {
    page(
        Theme::Light,
        Some(Anchor::Point(Point {
            x: Px(220.0),
            y: Px(120.0),
        })),
    )
}

fn started(app: fn() -> Element) -> Harness {
    let mut harness = Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(400));
    harness
}

#[test]
fn a_pop_up_that_starts_open_draws_its_menu_under_its_button() {
    for (app, shot) in [(Light as fn() -> Element, "light"), (Dark, "dark")] {
        let mut harness = started(app);
        let frame = harness.render().expect("renders");
        keep(&frame, &format!("pop-up-start-{shot}"));
        assert_eq!(harness.count(".ds-menu"), 1, "{shot}: the menu is up");
        let button = rect(&harness, ".ds-popup .ds-button");
        let menu = rect(&harness, ".ds-menu");
        assert!(
            menu.origin.y.0 >= button.origin.y.0 && menu.origin.x.0 < 120.0 + button.size.width.0,
            "{shot}: the menu sits at the button, {menu:?} against {button:?}"
        );
    }
}

#[test]
fn an_explicit_anchor_places_the_menu_there() {
    let harness = started(Placed);
    let menu = rect(&harness, ".ds-menu");
    assert!(
        menu.origin.x.0 >= 200.0,
        "the menu opens at the anchor's x: {menu:?}"
    );
}
