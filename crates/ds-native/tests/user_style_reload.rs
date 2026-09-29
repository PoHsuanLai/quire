//! A `<style>` whose text comes from a signal restyles the document when the signal changes
//! (ARCHITECTURE.md section 11, "User styles": the reload path is verified in Blitz). A button
//! steps the signal through red, green, a custom-property override, a changed override, and an
//! empty stylesheet; the probe's computed fill and its painted pixel follow each step.

use dioxus::prelude::*;
use ds::Point;
use ds_native::{Clock, Harness, HarnessConfig, Part, Srgba, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 200,
    height: 200,
    scale_percent: 100,
};

const STEPS: [&str; 5] = [
    ".probe { background: rgb(255,0,0); width: 60px; height: 60px }",
    ".probe { background: rgb(0,255,0); width: 60px; height: 60px }",
    ".probe { background: var(--x); width: 60px; height: 60px } .ds { --x: rgb(0,0,255) }",
    ".probe { background: var(--x); width: 60px; height: 60px } .ds { --x: rgb(255,255,0) }",
    "",
];

#[allow(non_snake_case)]
fn App() -> Element {
    let mut step = use_signal(|| 0usize);
    let text = STEPS[step()];
    rsx! {
        div { class: "ds",
            style { "{text}" }
            button { class: "next", onclick: move |_| step += 1, "next" }
            div { class: "probe" }
        }
    }
}

fn harness() -> Harness {
    Harness::with_config(App, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

fn next(harness: &mut Harness) {
    let at = harness.centre(".next").expect("the button is on screen");
    harness.click(at);
    harness.advance(Duration::from_millis(50));
}

fn fill(harness: &Harness) -> [u8; 4] {
    let Srgba(c) = harness
        .fill_of(".probe", Part::Element)
        .expect("probe fill");
    c.map(|v| (v * 255.0).round() as u8)
}

/// The painted pixel in the middle of the probe.
fn pixel(harness: &mut Harness) -> [u8; 4] {
    let at: Point = harness.centre(".probe").expect("probe on screen");
    let image = harness.render().expect("paint");
    image.get_pixel(at.x.0 as u32, at.y.0 as u32).0
}

#[test]
fn a_changed_style_text_restyles_the_probe() {
    let mut harness = harness();
    assert_eq!(fill(&harness), [255, 0, 0, 255], "initial red");
    assert_eq!(pixel(&mut harness), [255, 0, 0, 255], "initial red pixel");
    next(&mut harness);
    assert_eq!(fill(&harness), [0, 255, 0, 255], "green after the change");
    assert_eq!(pixel(&mut harness), [0, 255, 0, 255], "green pixel");
}

#[test]
fn a_changed_custom_property_override_restyles_the_probe() {
    let mut harness = harness();
    next(&mut harness);
    next(&mut harness);
    assert_eq!(fill(&harness), [0, 0, 255, 255], "var override blue");
    next(&mut harness);
    assert_eq!(fill(&harness), [255, 255, 0, 255], "var override yellow");
}

#[test]
fn removing_every_rule_returns_to_the_default() {
    let mut harness = harness();
    for _ in 0..4 {
        next(&mut harness);
    }
    assert_eq!(fill(&harness), [0, 0, 0, 0], "transparent again");
    assert_eq!(harness.rect(".probe").expect("probe").size.height.0, 0.0);
}
