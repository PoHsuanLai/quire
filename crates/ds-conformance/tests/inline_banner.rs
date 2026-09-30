//! `InlineBanner` on a real Blitz document: it sits in the pane's flow above what follows it,
//! its action and its close button each report one press, and its severity picks the mark's
//! colour.

use dioxus::prelude::*;
use ds::components::overlays::inline_banner::InlineBanner;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;
use ds_harness::{Driver, Harness, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 320,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// A banner over a paragraph; each press is counted in `.log`.
#[allow(non_snake_case)]
fn Pane() -> Element {
    let mut log = use_signal(Vec::<&'static str>::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "width:480px",
                InlineBanner {
                    text: "Remote images are blocked.",
                    detail: "Loading them tells the sender you opened this.",
                    actions: rsx! {
                        Button { label: "Load images", size: ControlSize::Small,
                            onclick: move |_| log.with_mut(|log| log.push("load")) }
                    },
                    onclose: move |()| log.with_mut(|log| log.push("close")),
                }
                p { class: "body", "The message" }
                InlineBanner { severity: Severity::Danger, text: "Not from who it says." }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

#[test]
fn the_banner_takes_its_place_in_the_flow_and_holds_its_width() {
    let mut harness = Harness::new(Pane, VIEW);
    harness.advance(ms(100));
    let banner = harness.rect(".ds-inline-banner").expect("the banner");
    let body = harness.rect(".body").expect("the message");
    assert!(
        body.origin.y.0 >= banner.origin.y.0 + banner.size.height.0 - 1.0,
        "the message starts under the banner: {banner:?} {body:?}"
    );
    assert!(
        (banner.size.width.0 - 480.0).abs() < 1.0,
        "the banner fills the pane: {banner:?}"
    );
}

#[test]
fn the_action_and_the_close_button_each_report_one_press() {
    let mut harness = Harness::new(Pane, VIEW);
    harness.advance(ms(100));
    assert_eq!(harness.text_of(".log").as_deref(), Some(""));
    let load = harness
        .centre(".ds-inline-banner-actions .ds-button")
        .expect("the action");
    harness.send(Input::click(load));
    harness.advance(ms(50));
    assert_eq!(harness.text_of(".log").as_deref(), Some("load"));
    let close = harness
        .centre(".ds-inline-banner > .ds-button")
        .expect("the close button");
    harness.send(Input::click(close));
    harness.advance(ms(50));
    assert_eq!(harness.text_of(".log").as_deref(), Some("load,close"));
}

#[test]
fn a_severity_colours_the_mark_and_a_danger_banner_is_an_alert() {
    let harness = {
        let mut harness = Harness::new(Pane, VIEW);
        harness.advance(ms(100));
        harness
    };
    let info = harness.ink_of("[data-severity=info] .ds-inline-banner-icon");
    let danger = harness.ink_of("[data-severity=danger] .ds-inline-banner-icon");
    assert!(info.is_some() && danger.is_some());
    assert_ne!(info, danger, "the mark's colour follows the severity");
    assert_eq!(
        harness.attr("[data-severity=danger]", "role").as_deref(),
        Some("alert")
    );
    assert_eq!(
        harness.attr("[data-severity=info]", "role").as_deref(),
        Some("status")
    );
}
