//! A menu anchored to a `Button` through its `mounted` handle (`Anchor::Mounted`) opens below
//! that button, on a real Blitz document: the handle is the button element itself, not a
//! wrapper, and the menu reads its rect after layout.

use dioxus::prelude::*;
use ds::host::measure::{Anchor, MountedRef};
use ds::prelude::*;
use ds::root::common::Common;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 360,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn AnchorApp() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            // Room on the left so the menu is not clamped to the 8 px margin. The
            // spacer gives height: the overlay bounds are the root's box, as tall as its
            // content, and a content-high root has no room below the button, so the menu flips.
            div { style: "padding:40px 0 0 60px",
                Anchored {}
            }
            div { style: "height:240px" }
        }
    }
}

#[allow(non_snake_case)]
fn Anchored() -> Element {
    let mut open = use_signal(|| Check::Off);
    let mut element = use_signal(|| None::<MountedRef>);
    let entries = ["Later today", "Tomorrow"]
        .into_iter()
        .zip(0u8..)
        .map(|(title, value)| MenuItem::Item {
            value,
            title: title.into(),
            image: None,
            key: None,
            check: None,
            availability: Availability::Enabled,
            hint: None,
            after: AfterPick::Close,
        })
        .collect::<Vec<_>>();
    rsx! {
        Button {
            common: Common { mounted: Some(EventHandler::new(move |event: MountedEvent| element.set(Some(MountedRef(event.data()))))), ..Common::default() },
            label: "Snooze",
            onclick: move |_| open.set(Check::On),
        }
        // Something after the button: with only the `if` placeholder after it, the harness's
        // click never reached the button (FINDINGS "Polish pass").
        p { "Snooze this thread" }
        if let (Check::On, Some(button)) = (open(), element()) {
            Menu {
                placement: MenuPlacement::Popup,
                anchor: Anchor::Mounted(button),
                items: entries,
                onpick: move |_: u8| open.set(Check::Off),
                onclose: move |_| open.set(Check::Off),
            }
        }
    }
}

fn rect(harness: &Harness, selector: &str) -> Rect {
    harness
        .rect(selector)
        .unwrap_or_else(|| panic!("{selector} is not in the document:\n{}", harness.html()))
}

#[test]
fn a_menu_anchored_to_a_buttons_mounted_handle_opens_below_it() {
    let mut harness = Harness::new(
        AnchorApp,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    let button = rect(&harness, ".ds-button");
    let at = harness
        .centre(".ds-button")
        .expect("the button is on screen");
    harness.send(Input::click(at));
    // Frames for the anchor's rect read (a task that waits a frame, then a render), and the
    // entrance to settle.
    for _ in 0..10 {
        harness.advance(Duration::from_millis(100));
    }
    assert_eq!(harness.count(".ds-menu"), 1, "{}", harness.html());
    let menu = rect(&harness, ".ds-menu");
    let bottom = button.origin.y.0 + button.size.height.0;
    // A pop-up menu hangs 2 below its anchor, on the anchor's left edge (design/30 section 2.4).
    let near = |got: f32, want: f32| (got - want).abs() <= 1.0;
    assert!(
        near(menu.origin.y.0, bottom + 2.0),
        "the menu's top {} is not 2 below the button's bottom {bottom}",
        menu.origin.y.0
    );
    assert!(
        near(menu.origin.x.0, button.origin.x.0),
        "the menu's left {} is not the button's {}",
        menu.origin.x.0,
        button.origin.x.0
    );
    assert!(
        button.origin.y.0 > 30.0,
        "the button sits in its padding: {button:?}"
    );
}
