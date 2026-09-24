//! A menu anchored to a `Button` through its `mounted` handle (`Anchor::Mounted`) opens below
//! that button, on a real Blitz document: the handle is the button element itself, not a
//! wrapper, and the menu reads its rect after layout.

use dioxus::prelude::*;
use ds::{
    Anchor, Appearance, Button, ButtonVariant, Ds, Material, Menu, MenuEntry, MenuKind, MountedRef,
    Rect, Switch, Trail,
};
use ds_native::{Harness, Viewport};
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
            // Room on the left so the menu's `left - 8` is not clamped to the 8 px margin. The
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
    let mut open = use_signal(|| Switch::Off);
    let mut element = use_signal(|| None::<MountedRef>);
    let entries = ["Later today", "Tomorrow"]
        .into_iter()
        .zip(0u8..)
        .map(|(title, value)| MenuEntry::Item {
            value,
            title: title.into(),
            detail: None,
            tile: None,
            trail: Trail::None,
            check: None,
        })
        .collect::<Vec<_>>();
    rsx! {
        Button {
            variant: ButtonVariant::Secondary,
            label: "Snooze",
            onclick: move |_| open.set(Switch::On),
            mounted: move |event: MountedEvent| element.set(Some(MountedRef(event.data()))),
        }
        // Something after the button: with only the `if` placeholder after it, the harness's
        // click never reached the button (FINDINGS "Gallery fixes B").
        p { "Snooze this thread" }
        if let (Switch::On, Some(button)) = (open(), element()) {
            Menu {
                kind: MenuKind::Slim,
                anchor: Anchor::Mounted(button),
                entries,
                onpick: move |_: u8| open.set(Switch::Off),
                onclose: move |_| open.set(Switch::Off),
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
    let mut harness = Harness::new(AnchorApp, VIEW);
    let button = rect(&harness, ".ds-button");
    let at = harness
        .centre(".ds-button")
        .expect("the button is on screen");
    harness.click(at);
    // Frames for the anchor's rect read (a task that waits a frame, then a render), and the
    // entrance to settle.
    for _ in 0..10 {
        harness.advance(Duration::from_millis(100));
    }
    assert_eq!(harness.count(".ds-menu"), 1, "{}", harness.html());
    let menu = rect(&harness, ".ds-menu");
    let bottom = button.origin.y.0 + button.size.height.0;
    // Slim menus sit 6 below the anchor and 8 left of it (design/04-COMPONENTS.md section 20).
    let near = |got: f32, want: f32| (got - want).abs() <= 1.0;
    assert!(
        near(menu.origin.y.0, bottom + 6.0),
        "the menu's top {} is not 6 below the button's bottom {bottom}",
        menu.origin.y.0
    );
    assert!(
        near(menu.origin.x.0, button.origin.x.0 - 8.0),
        "the menu's left {} is not 8 left of the button's {}",
        menu.origin.x.0,
        button.origin.x.0
    );
    assert!(
        button.origin.y.0 > 30.0,
        "the button sits in its padding: {button:?}"
    );
}
