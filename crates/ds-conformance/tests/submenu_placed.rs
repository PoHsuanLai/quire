//! A submenu is never painted before it is placed: it stays hidden until its target and its own
//! size are measured, then lands beside its parent row, whether its menu floats or is drawn
//! inline in the caller's card.

use dioxus::prelude::*;
use ds::host::measure::Anchor;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 360,
    scale_percent: 100,
};

const SUBMENU: &str = ".ds-menu[*|data-depth]";
const ROOT_MENU: &str = ".ds-menu:not([*|data-depth]), .g-card";
const PARENT: &str = ".ds-menu-item[*|aria-haspopup=true]";

fn item(value: u8, title: &str) -> MenuItem<u8> {
    MenuItem::Item {
        value,
        title: title.to_string(),
        image: None,
        key: None,
        check: None,
        availability: Availability::Enabled,
        hint: None,
        after: AfterPick::Close,
    }
}

fn entries() -> Vec<MenuItem<u8>> {
    vec![
        item(1, "Open"),
        item(3, "Quit"),
        MenuItem::Submenu {
            title: "More".to_string(),
            image: None,
            availability: Availability::Enabled,
            children: vec![item(10, "About"), item(11, "Help")],
        },
    ]
}

#[allow(non_snake_case)]
fn Floating() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:340px" }
            Menu::<u8> {
                placement: MenuPlacement::Popup,
                anchor: Anchor::Point(Point { x: Px(40.0), y: Px(60.0) }),
                items: entries(),
                expanded: Some(2),
                onpick: |_| {},
                onclose: |_| {},
            }
        }
    }
}

#[allow(non_snake_case)]
fn Inline() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:340px; padding:40px",
                div { class: "g-card", style: "width:200px",
                    Menu::<u8> {
                        placement: MenuPlacement::Popup,
                        anchor: Anchor::Point(Point::default()),
                        items: entries(),
                        flow: Flow::Inline,
                        expanded: Some(2),
                        onpick: |_| {},
                        onclose: |_| {},
                    }
                }
            }
        }
    }
}

/// Runs `app`, watching every frame: a submenu that is painted (not hidden) is already beside
/// its parent row, never at the origin. Once it settles it sits right of the menu, its top level
/// with the parent row's.
fn watch(app: fn() -> Element) {
    let mut harness = Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    let mut seen = false;
    for _ in 0..120 {
        harness.advance(Duration::from_millis(16));
        if harness.count(SUBMENU) == 0 {
            continue;
        }
        seen = true;
        let hidden = harness
            .attr(SUBMENU, "style")
            .is_some_and(|style| style.contains("visibility:hidden"));
        let sub = harness.rect(SUBMENU).expect("the submenu is laid out");
        let parent = harness.rect(PARENT).expect("the parent row");
        if !hidden {
            assert!(
                sub.origin.x.0 > parent.origin.x.0 && sub.origin.y.0 > 0.0,
                "a painted submenu is placed, not at the origin: {sub:?} for {parent:?}"
            );
        }
    }
    assert!(seen, "the submenu opened:\n{}", harness.html());
    let sub = harness.rect(SUBMENU).expect("the submenu");
    let parent = harness.rect(PARENT).expect("the parent row");
    let menu = harness.rect(ROOT_MENU).expect("the menu");
    assert!(
        harness
            .attr(SUBMENU, "style")
            .is_some_and(|style| !style.contains("visibility:hidden")),
        "it is shown once placed"
    );
    assert!(
        (sub.origin.x.0 - (menu.origin.x.0 + menu.size.width.0 + 2.0)).abs() <= 1.0
            || (sub.origin.x.0 + sub.size.width.0 + 2.0 - menu.origin.x.0).abs() <= 1.0,
        "beside the menu: {sub:?} for {menu:?}"
    );
    assert!(
        (sub.origin.y.0 - (parent.origin.y.0 - 5.0)).abs() <= 1.0,
        "level with the parent row: {sub:?} for {parent:?}"
    );
}

#[test]
fn a_floating_submenu_is_hidden_until_placed_then_lands_beside_its_parent_row() {
    watch(Floating);
}

#[test]
fn an_inline_submenu_is_hidden_until_placed_then_lands_beside_its_parent_row() {
    watch(Inline);
}
