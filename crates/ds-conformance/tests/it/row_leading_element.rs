//! A row led by an element (`RowLeading::Element`) stays one hit target: a click on the element
//! fires the row's onclick, as a click on its title does.

use dioxus::prelude::*;
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 320,
    height: 120,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Led() -> Element {
    let mut opened = use_signal(|| 0u32);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "width:260px; padding:8px",
                Row {
                    title: "main",
                    leading: RowLeading::Element(rsx! {
                        span { class: "orb", style: "display:block; width:10px; height:10px" }
                    }),
                    onclick: move |_| opened += 1,
                }
            }
            p { class: "log", "{opened}" }
        }
    }
}

#[test]
fn a_click_on_the_leading_element_opens_the_row() {
    let mut harness = Harness::new(Led, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    assert_eq!(harness.text_of(".log").as_deref(), Some("0"));
    harness.send(Input::click(harness.centre(".orb").expect("the orb")));
    harness.advance(Duration::from_millis(30));
    assert_eq!(harness.text_of(".log").as_deref(), Some("1"));
}
