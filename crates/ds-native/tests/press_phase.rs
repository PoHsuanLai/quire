//! design/30 section 1.4, Press: `data-pressed` is written on a Button (image-only included) from the
//! pointer going down (primary only) or Space or Return going down until it is released, and
//! is gone once the press ends.

use dioxus::prelude::*;
use ds::{Appearance, Button, Ds, Icon, Material, ShortcutKey};
use ds::{Bezel, ControlSize, ImagePosition};
use ds_native::{Harness, Viewport};

const VIEW: Viewport = Viewport {
    width: 240,
    height: 120,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            Button { label: "Save", onclick: move |_| {} }
            Button { bezel: Bezel::Toolbar, size: ControlSize::Large, image: ImagePosition::Only, icon: Icon::Plus, label: "Add", onclick: move |_| {} }
        }
    }
}

fn pressed(harness: &Harness, selector: &str) -> Option<String> {
    harness.attr(selector, "data-pressed")
}

#[test]
fn a_pointer_down_presses_a_button_until_it_comes_up() {
    let mut harness = Harness::new(Page, VIEW);
    let at = harness.centre(".ds-button").expect("the button");
    assert_eq!(pressed(&harness, ".ds-button"), None);
    harness.pointer_move(at);
    harness.pointer_down(at);
    assert_eq!(pressed(&harness, ".ds-button").as_deref(), Some("true"));
    harness.pointer_up(at);
    assert_eq!(pressed(&harness, ".ds-button"), None);
}

#[test]
fn a_secondary_button_does_not_press() {
    let mut harness = Harness::new(Page, VIEW);
    let at = harness.centre(".ds-button").expect("the icon button");
    harness.pointer_move(at);
    harness.button_down(at, ds::PointerButton::Secondary);
    assert_eq!(pressed(&harness, ".ds-button"), None);
}

#[test]
fn space_presses_the_focused_button() {
    let mut harness = Harness::new(Page, VIEW);
    harness.key(ShortcutKey::Tab);
    harness.key(ShortcutKey::Space);
    assert_eq!(
        pressed(&harness, ".ds-button"),
        None,
        "released by the key coming up"
    );
}
