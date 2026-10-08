//! A secondary press over a surface Blitz forwards its pointer events to (a texture layer, a
//! frame's document) or over selectable text that the hand drifts across still reaches an
//! ancestor's `oncontextmenu`: the menu an app opens over its picture, its rendered markdown or
//! its text. These are anyview's right-click cases, without the app.

use dioxus::prelude::*;
use ds::base::press::PointerButton;
use ds::prelude::{Point, Px};
use ds_blitz::TextureLayer;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 300,
    scale_percent: 100,
};

const FRAME_BODY: &str = "<body style='margin:0; font-size:30px'>Rendered markdown</body>";

/// A zone that counts the context menus asked of it, around `surface`.
#[component]
fn Zone(surface: Element) -> Element {
    let mut asked = use_signal(|| 0u32);
    rsx! {
        div {
            class: "zone",
            style: "width: 300px; height: 200px",
            oncontextmenu: move |_| asked += 1,
            {surface}
        }
        for _ in 0..asked() {
            i { class: "asked" }
        }
    }
}

#[allow(non_snake_case)]
fn Layer() -> Element {
    rsx! { Zone { surface: rsx! {
        div { style: "width: 300px; height: 200px", TextureLayer {} }
    } } }
}

#[allow(non_snake_case)]
fn Frame() -> Element {
    rsx! { Zone { surface: rsx! {
        iframe { class: "page", srcdoc: FRAME_BODY, style: "width: 300px; height: 200px; border: 0" }
    } } }
}

#[allow(non_snake_case)]
fn Text() -> Element {
    rsx! { Zone { surface: rsx! {
        p { style: "margin: 0; font-size: 30px", "Selectable words that a hand drifts across" }
    } } }
}

fn harness(app: fn() -> Element) -> Harness {
    Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

fn at(x: f32, y: f32) -> Point {
    Point { x: Px(x), y: Px(y) }
}

/// A secondary press at `from` that moves `drift` px before it is released, then how many
/// context menus the zone was asked for.
fn right_click(harness: &mut Harness, from: Point, drift: f32) -> usize {
    let to = at(from.x.0 + drift, from.y.0 + drift);
    harness.send(Input::pointer_move(from));
    harness.send(Input::button_down(from, PointerButton::Secondary));
    if drift != 0.0 {
        harness.send(Input::pointer_move(to));
    }
    harness.send(Input::button_up(to, PointerButton::Secondary));
    harness.advance(Duration::from_millis(50));
    harness.count(".asked")
}

#[test]
fn a_right_click_over_a_texture_layer_asks_for_the_menu() {
    let mut harness = harness(Layer);
    assert_eq!(right_click(&mut harness, at(150.0, 100.0), 0.0), 1);
}

#[test]
fn a_right_click_over_a_frame_asks_for_the_menu() {
    let mut harness = harness(Frame);
    assert_eq!(right_click(&mut harness, at(150.0, 100.0), 0.0), 1);
}

#[test]
fn a_right_click_that_drifts_over_selectable_text_asks_for_the_menu() {
    let mut harness = harness(Text);
    assert_eq!(right_click(&mut harness, at(50.0, 15.0), 3.0), 1);
}

#[test]
fn a_right_click_that_drifts_over_a_texture_layer_asks_for_the_menu() {
    let mut harness = harness(Layer);
    assert_eq!(right_click(&mut harness, at(150.0, 100.0), 3.0), 1);
}
