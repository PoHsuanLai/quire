//! A registered custom property (`@property`) animates on the pinned Blitz in the headless
//! harness: `--angle` keyframed from 0 to 360deg turns a conic gradient exactly as a `rotate()`
//! keyframe turns the same gradient, sampled on the virtual clock. The voice orb still drives its
//! turn from Rust, because quire's rules allow no infinite CSS loop (lint `InfiniteLoop`), only a
//! bounded Rust one that exists while its state holds (design/30 section 1.3); this test keeps
//! the fact, measured only on the headless path, not in a window.

use dioxus::prelude::*;
use ds_blitz::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 300,
    height: 130,
    scale_percent: 100,
};

const QUADRANTS: &str =
    "rgb(255,0,0) 0 25%, rgb(0,255,0) 25% 50%, rgb(0,0,255) 50% 75%, rgb(255,255,0) 75% 100%";

fn page() -> Element {
    rsx! {
        style {
            "@property --angle {{ syntax: '<angle>'; inherits: false; initial-value: 0deg; }} \
             @keyframes property-turn {{ to {{ --angle: 360deg; }} }} \
             @keyframes transform-turn {{ to {{ transform: rotate(360deg); }} }} \
             #property {{ animation: property-turn 4s linear infinite; }} \
             #transform {{ animation: transform-turn 4s linear infinite; }}"
        }
        div { style: "position:absolute; left:0; top:0; width:300px; height:130px; background:#fff;",
            div { id: "property", style: "position:absolute; left:10px; top:10px; width:100px; height:100px; background:conic-gradient(from var(--angle) at 50% 50%, {QUADRANTS});" }
            div { id: "transform", style: "position:absolute; left:150px; top:10px; width:100px; height:100px; background:conic-gradient({QUADRANTS});" }
        }
    }
}

const RED: [u8; 3] = [255, 0, 0];
const YELLOW: [u8; 3] = [255, 255, 0];

/// The pixel 30 px from a square's centre at 60 degrees clockwise from the top, which the
/// gradient paints red at rest and yellow once turned a quarter.
fn probe(harness: &mut Harness, id: &str) -> [u8; 3] {
    let frame = harness.render().expect("paints");
    let at = harness.rect(&format!("#{id}")).expect("laid out");
    let p = frame.get_pixel(at.origin.x.0 as u32 + 76, at.origin.y.0 as u32 + 35);
    [p[0], p[1], p[2]]
}

#[test]
fn an_animated_registered_property_turns_a_conic_gradient_like_a_rotation() {
    let mut harness =
        Harness::with_config(page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    let mut seen = Vec::new();
    for pause in [0, 500, 500] {
        harness.advance(Duration::from_millis(pause));
        seen.push((
            probe(&mut harness, "property"),
            probe(&mut harness, "transform"),
        ));
    }
    // At rest, an eighth and a quarter of the way round: red, red, then yellow, for both.
    assert_eq!(
        seen,
        vec![(RED, RED), (RED, RED), (YELLOW, YELLOW)],
        "property turn vs rotate() turn"
    );
}
