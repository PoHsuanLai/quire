//! A `SectionHeader` with two actions on a real Blitz document: both end the row, side by side
//! and in order, and each press reaches its own handler.

use dioxus::prelude::*;
use ds::components::lists::section_header::HeaderAction;
use ds::prelude::*;
use ds_harness::{Driver, Harness, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 320,
    height: 80,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[allow(non_snake_case)]
fn Header() -> Element {
    let mut log = use_signal(Vec::<&'static str>::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "width:300px",
                SectionHeader {
                    title: "Labels",
                    actions: vec![
                        HeaderAction::new("Select All", EventHandler::new(move |()| log.with_mut(|l| l.push("all")))),
                        HeaderAction::new("Clear", EventHandler::new(move |()| log.with_mut(|l| l.push("clear")))),
                    ],
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

#[test]
fn two_actions_end_the_row_in_order_and_each_hears_its_own_press() {
    let mut harness = Harness::new(Header, VIEW);
    harness.advance(ms(100));
    let first = harness
        .rect(".ds-section-header-action:nth-child(1)")
        .expect("first");
    let second = harness
        .rect(".ds-section-header-action:nth-child(2)")
        .expect("second");
    let header = harness.rect(".ds-section-header").expect("header");
    assert!(
        first.origin.x.0 + first.size.width.0 <= second.origin.x.0,
        "side by side, in order: {first:?} {second:?}"
    );
    let header_end = header.origin.x.0 + header.size.width.0;
    let end = second.origin.x.0 + second.size.width.0;
    assert!(
        header_end - end < 12.0,
        "the last action ends the row: {end} of {header_end}"
    );
    harness.send(Input::click(
        harness
            .centre(".ds-section-header-action:nth-child(2)")
            .expect("clear"),
    ));
    harness.advance(ms(50));
    assert_eq!(harness.text_of(".log").as_deref(), Some("clear"));
    harness.send(Input::click(
        harness
            .centre(".ds-section-header-action:nth-child(1)")
            .expect("all"),
    ));
    harness.advance(ms(50));
    assert_eq!(harness.text_of(".log").as_deref(), Some("clear,all"));
}
