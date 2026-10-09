//! A tooltip keyed by the caller's own pointer hooks (design/30 section 2.5), on a real Blitz
//! document: a thread row's time feeds the hover hub from its own events through
//! `use_hover_intent` and draws `Tooltip { hover_key }` wrapping nothing. The tip opens at once,
//! stands below the element it was anchored to, and goes when the
//! pointer leaves.

use dioxus::prelude::*;
use ds::components::overlays::hover_card::intent::{HoverAnchor, use_hover_intent};
use ds::host::measure::MountedRef;
use ds::motion::hover_intent::HoverProfile;
use ds::prelude::*;
use ds::stack::hover_hub::HoverKey;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 400,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// A row whose time names its full date on hover, by the row's own hooks.
#[component]
fn Time() -> Element {
    let driver = use_hover_intent();
    let mut element = use_signal(|| None::<MountedRef>);
    let key = HoverKey("time:1".to_string());
    let tip_key = key.clone();
    rsx! {
        div { style: "position:absolute;left:300px;top:120px",
            span {
                id: "time",
                style: "display:inline-block;width:60px;height:18px",
                onmounted: move |event| element.set(Some(MountedRef(event.data()))),
                onpointerenter: {
                    let key = key.clone();
                    move |_| {
                        let anchor = element
                            .peek()
                            .clone()
                            .map_or(HoverAnchor::Unplaced, HoverAnchor::Element);
                        driver.over(key.clone(), HoverProfile::Tip, anchor);
                    }
                },
                onpointerleave: move |_| driver.out(),
                "3:42 PM"
            }
            Tooltip { text: "Thursday, October 1, 2026 at 3:42 PM", hover_key: Some(tip_key) }
        }
    }
}

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            Time {}
        }
    }
}

#[allow(non_snake_case)]
fn Dark() -> Element {
    rsx! {
        Ds { appearance: Appearance { theme: Theme::Dark, ..Appearance::default() }, material: Material::Window, extent: RootExtent::Viewport,
            Time {}
        }
    }
}

fn started() -> Harness {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(200));
    harness
}

#[test]
fn the_tip_opens_at_once_below_its_anchor_and_goes_with_the_pointer() {
    let mut harness = started();
    assert_eq!(
        harness.count(".ds-tooltip"),
        0,
        "nothing before the pointer"
    );
    assert_eq!(
        harness.count(".ds-hover-target"),
        0,
        "the caller's element is not wrapped"
    );
    let over = harness.centre("#time").expect("the time");
    harness.send(Input::pointer_move(over));
    harness.advance(ms(50));
    assert_eq!(harness.count(".ds-tooltip"), 1, "no wait by default");
    let tip = harness.rect(".ds-tooltip").expect("the tip");
    let time = harness.rect("#time").expect("the time");
    assert!(
        (tip.origin.y.0 - (time.origin.y.0 + time.size.height.0)).abs() < 12.0,
        "it stands below the anchor: {tip:?} {time:?}"
    );
    assert_eq!(
        harness.attr(".ds-tooltip", "role").as_deref(),
        Some("tooltip")
    );
    harness.send(Input::pointer_move(Point {
        x: Px(20.0),
        y: Px(380.0),
    }));
    harness.advance(ms(600));
    assert_eq!(
        harness.count(".ds-tooltip"),
        0,
        "gone once the pointer left"
    );
}

/// Each scheme's picture with the tip up paints; it is written only when `QUIRE_SHOTS` names a
/// directory, as `tooltip-hooked-<scheme>.png`, for the progress gallery.
#[test]
fn the_hooked_tip_paints_in_both_schemes() {
    let out = std::env::var("QUIRE_SHOTS").ok();
    for (app, scheme) in [(Page as fn() -> Element, "light"), (Dark, "dark")] {
        let mut harness = Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
        harness.advance(ms(200));
        let over = harness.centre("#time").expect("the time");
        harness.send(Input::pointer_move(over));
        harness.advance(ms(1300));
        assert_eq!(harness.count(".ds-tooltip"), 1, "{scheme}");
        let shot = harness.render().expect("renders");
        if let Some(dir) = &out {
            shot.save(format!("{dir}/tooltip-hooked-{scheme}.png"))
                .expect("writes the shot");
        }
    }
}
