//! A PopUpButton opened in a document with no renderer (a `VirtualDom` and `dioxus-ssr`, the
//! way an app's unit tests build one): a renderer reports the button's element after layout and
//! this document never does, so the menu opens against `anchor` instead, from `start` or from
//! the first click's state. Without either the menu stays closed, as it does before layout.

use dioxus::prelude::*;
use ds::components::menus::pop_up_button::{PopUpButton, PopUpKind};
use ds::host::measure::Anchor;
use ds::prelude::*;

fn items() -> Vec<MenuItem<u8>> {
    vec![
        MenuItem::new(1, "New folder inside"),
        MenuItem::new(2, "Rename"),
    ]
}

#[allow(non_snake_case)]
fn Closed() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            PopUpButton::<u8> { items: items(), onpick: |_| {}, kind: PopUpKind::Overflow }
        }
    }
}

#[allow(non_snake_case)]
fn NoAnchor() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            PopUpButton::<u8> { items: items(), onpick: |_| {}, kind: PopUpKind::Overflow, start: Shown::Visible }
        }
    }
}

#[allow(non_snake_case)]
fn Open() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            PopUpButton::<u8> {
                items: items(),
                onpick: |_| {},
                kind: PopUpKind::Overflow,
                start: Shown::Visible,
                anchor: Some(Anchor::Point(Point { x: Px(40.0), y: Px(20.0) })),
            }
        }
    }
}

/// Render `app` and let the root's overlay take what was asked of it, as an app's test would.
fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    for _ in 0..8 {
        dom.process_events();
        dom.render_immediate(&mut dioxus_core::NoOpMutations);
    }
    dioxus_ssr::render(&dom)
}

#[test]
fn a_pop_up_opens_against_an_explicit_anchor_with_no_renderer() {
    let html = render(Open);
    assert!(html.contains("class=\"ds-menu"), "{html}");
    for item in ["New folder inside", "Rename"] {
        assert!(html.contains(item), "{item} missing: {html}");
    }
}

#[test]
fn a_pop_up_stays_shut_by_default_and_without_a_place_to_open() {
    assert!(!render(Closed).contains("class=\"ds-menu"));
    assert!(
        !render(NoAnchor).contains("class=\"ds-menu"),
        "no element and no anchor: nothing to place the menu against"
    );
}
