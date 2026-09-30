//! design/30 section 1.4, Press: `data-pressed` is written on a Button (image-only included) from the
//! pointer going down (primary only) or Space or Return going down until it is released, and
//! is gone once the press ends.

use dioxus::prelude::*;
use ds::components::controls::button_model::{Bezel, ImagePosition};
use ds::prelude::*;
use ds_core::press::PointerButton;
use ds_harness::{Driver, Harness, Input, Query, Viewport};
use ds_style::tokens::control_size::ControlSize;

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
    harness.send(Input::pointer_move(at));
    harness.send(Input::pointer_down(at));
    assert_eq!(pressed(&harness, ".ds-button").as_deref(), Some("true"));
    harness.send(Input::pointer_up(at));
    assert_eq!(pressed(&harness, ".ds-button"), None);
}

#[test]
fn a_secondary_button_does_not_press() {
    let mut harness = Harness::new(Page, VIEW);
    let at = harness.centre(".ds-button").expect("the icon button");
    harness.send(Input::pointer_move(at));
    harness.send(Input::button_down(at, PointerButton::Secondary));
    assert_eq!(pressed(&harness, ".ds-button"), None);
}

#[test]
fn space_presses_the_focused_button() {
    let mut harness = Harness::new(Page, VIEW);
    harness.send(Input::key(ShortcutKey::Tab));
    harness.send(Input::key(ShortcutKey::Space));
    assert_eq!(
        pressed(&harness, ".ds-button"),
        None,
        "released by the key coming up"
    );
}
