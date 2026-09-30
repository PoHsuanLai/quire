//! Blitz's Stylo honours `@layer`: an unlayered rule beats a layered rule whatever the
//! specificity, and a later layer beats an earlier one (ARCHITECTURE.md section 10).

use dioxus::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Part, Query, Srgba, Viewport};

const VIEW: Viewport = Viewport {
    width: 100,
    height: 100,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Layered() -> Element {
    rsx! {
        div { class: "ds",
            style {
                "@layer a, b; \
                 @layer a {{ #one.probe.x {{ background: rgb(255,0,0); width: 20px; height: 20px }} \
                             #two.probe.x {{ background: rgb(255,0,0); width: 20px; height: 20px }} }} \
                 @layer b {{ .probe {{ background: rgb(0,0,255) }} }} \
                 #one {{ background: rgb(0,255,0) }}"
            }
            div { id: "one", class: "probe x" }
            div { id: "two", class: "probe x" }
        }
    }
}

fn fill(harness: &Harness, selector: &str) -> [u8; 4] {
    let Srgba(c) = harness.fill_of(selector, Part::Element).expect("fill");
    c.map(|v| (v * 255.0).round() as u8)
}

#[test]
fn unlayered_beats_layered_and_later_layer_beats_earlier() {
    let harness = Harness::new(Layered, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    assert_eq!(fill(&harness, "#one"), [0, 255, 0, 255], "unlayered wins");
    assert_eq!(fill(&harness, "#two"), [0, 0, 255, 255], "later layer wins");
}
