//! A `Label`'s severity on a real Blitz document: each status is its own colour, and none is
//! the ink of a label without one.

use dioxus::prelude::*;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 320,
    height: 160,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Labels() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "display:flex; flex-direction:column",
                Label { text: "plain", common: ds::root::common::Common { id: Some("plain".into()), ..Default::default() } }
                Label { text: "info", severity: Some(Severity::Info) }
                Label { text: "ok", severity: Some(Severity::Ok) }
                Label { text: "warn", severity: Some(Severity::Warn) }
                Label { text: "danger", severity: Some(Severity::Danger) }
            }
        }
    }
}

#[test]
fn each_severity_is_its_own_colour_and_none_is_the_plain_ink() {
    let mut harness = Harness::new(Labels, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(100));
    let plain = harness.ink_of("#plain").expect("plain ink");
    let inks: Vec<_> = Severity::ALL
        .iter()
        .map(|s| {
            harness
                .ink_of(&format!(".ds-label[data-severity={}]", s.slug()))
                .expect("a status ink")
        })
        .collect();
    for (index, ink) in inks.iter().enumerate() {
        assert_ne!(*ink, plain, "{:?} differs from plain", Severity::ALL[index]);
        assert!(
            inks.iter()
                .enumerate()
                .all(|(other, o)| other == index || o != ink),
            "{:?} is its own colour: {inks:?}",
            Severity::ALL[index]
        );
    }
}
