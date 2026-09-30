//! The app features on a real Blitz document (design/30 section 2.11): the edge-peek sidebar
//! reveals after the pointer rests on its strip and hides when it leaves, a click on the strip
//! pins it; a Today tab is dropped when its time is up; the link pill expands under the pointer
//! and hands its address over on a click.

use dioxus::prelude::*;
use ds::time::now;
use ds::{
    Appearance, Ds, EdgePeek, LinkPill, LinkTarget, Material, Point, Px, RowLeading, Shown,
    TodayTab, TodayTabs,
};
use ds_harness::{Driver, Harness, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 300,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[allow(non_snake_case)]
fn Edge() -> Element {
    let mut pinned = use_signal(|| Shown::Hidden);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "position:relative;width:400px;height:300px",
                EdgePeek { label: "Sidebar", pinned: pinned(), onpin: move |()| pinned.set(Shown::Visible), "Inbox" }
            }
        }
    }
}

fn side(harness: &Harness) -> Option<String> {
    harness.attr("nav.ds-side", "data-side")
}

#[test]
fn the_edge_peeks_the_sidebar_after_a_rest_and_hides_it_on_leaving() {
    let mut harness = Harness::new(Edge, VIEW);
    harness.advance(ms(50));
    assert_eq!(side(&harness).as_deref(), Some("hidden"));
    let edge = Point {
        x: Px(4.0),
        y: Px(150.0),
    };
    harness.send(Input::pointer_move(edge));
    harness.advance(ms(30));
    assert_eq!(
        side(&harness).as_deref(),
        Some("hidden"),
        "not before the intent delay"
    );
    harness.advance(ms(200));
    assert_eq!(side(&harness).as_deref(), Some("peek"));
    harness.advance(ms(700));
    let style = harness.attr("nav.ds-side", "style").unwrap_or_default();
    assert!(style.contains("--present-p:1"), "in place: {style}");
    harness.send(Input::pointer_move(Point {
        x: Px(350.0),
        y: Px(150.0),
    }));
    harness.advance(ms(1000));
    assert_eq!(
        side(&harness).as_deref(),
        Some("hidden"),
        "gone once the pointer left"
    );
}

#[test]
fn a_click_on_the_strip_pins_the_sidebar() {
    let mut harness = Harness::new(Edge, VIEW);
    harness.advance(ms(50));
    harness.send(Input::click(Point {
        x: Px(4.0),
        y: Px(150.0),
    }));
    harness.advance(ms(100));
    assert_eq!(side(&harness).as_deref(), Some("shown"));
    assert_eq!(harness.count(".ds-edge"), 0, "no strip while pinned");
}

#[allow(non_snake_case)]
fn Today() -> Element {
    let start = now();
    let tab = |key: u8, title: &str, secs: u64| TodayTab {
        key,
        title: title.to_string(),
        leading: RowLeading::None,
        expires: start + Duration::from_secs(secs),
    };
    let mut tabs = use_signal(move || vec![tab(1, "Keeps", 3 * 3600), tab(2, "Goes", 3)]);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            TodayTabs {
                label: "Today",
                tabs: tabs(),
                onpick: |_| {},
                onclose: move |key| tabs.with_mut(|tabs| tabs.retain(|tab| tab.key != key)),
                onexpire: move |key| tabs.with_mut(|tabs| tabs.retain(|tab| tab.key != key)),
            }
        }
    }
}

#[test]
fn a_today_tab_leaves_when_its_time_is_up() {
    let mut harness = Harness::new(Today, VIEW);
    harness.advance(ms(400));
    assert_eq!(harness.count(".ds-row"), 2);
    assert_eq!(
        harness
            .attr(".ds-row[*|data-shape=today-soon]", "aria-selected")
            .as_deref(),
        Some("false"),
        "the tab with seconds left is drawn quiet"
    );
    harness.advance(Duration::from_secs(4));
    harness.advance(ms(1000));
    assert_eq!(harness.count(".ds-row"), 1, "{}", harness.html());
}

#[allow(non_snake_case)]
fn Link() -> Element {
    let mut copied = use_signal(String::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "position:relative;width:400px;height:300px",
                LinkPill {
                    href: "https://www.acme.example/a/b",
                    oncopy: move |href| copied.set(href),
                    target: LinkTarget::Honest {
                        scheme_sub: "https://www.".to_string(),
                        registered: "acme.example".to_string(),
                        path: "/a/b".to_string(),
                    },
                }
            }
            p { class: "copied", "{copied()}" }
        }
    }
}

#[test]
fn the_link_pill_expands_under_the_pointer_and_copies_on_a_click() {
    let mut harness = Harness::new(Link, VIEW);
    harness.advance(ms(300));
    assert_eq!(
        harness.count(".ds-link-pill-dim"),
        0,
        "collapsed: the domain only"
    );
    let pill = harness.centre(".ds-link-pill").expect("the pill");
    harness.send(Input::pointer_move(pill));
    harness.advance(ms(400));
    assert_eq!(
        harness.attr(".ds-link-pill", "data-expanded").as_deref(),
        Some("true")
    );
    assert!(harness.count(".ds-link-pill-dim") >= 1, "the whole address");
    let pill = harness.centre(".ds-link-pill").expect("the pill");
    harness.send(Input::click(pill));
    harness.advance(ms(100));
    assert_eq!(
        harness.text_of(".copied").as_deref(),
        Some("https://www.acme.example/a/b")
    );
    assert_eq!(harness.text_of(".ds-link-pill").as_deref(), Some("Copied"));
}
