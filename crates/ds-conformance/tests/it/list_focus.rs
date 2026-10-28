//! `List { autofocus: true }` takes the keyboard as it mounts, so the arrow keys work
//! at once; the default leaves the keyboard where it was.

use dioxus::prelude::*;
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 320,
    height: 240,
    scale_percent: 100,
};

fn rows() -> Vec<ListItem<u8>> {
    (0..3u8)
        .map(|n| ListItem::row(n, format!("Row {n}"), rsx! { div { "Row {n}" } }))
        .collect()
}

#[allow(non_snake_case)]
fn Mounted() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            List::<u8> { label: "Applications", items: rows(), autofocus: true }
        }
    }
}

#[allow(non_snake_case)]
fn Manual() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            List::<u8> { label: "Applications", items: rows() }
        }
    }
}

fn harness(app: fn() -> Element) -> Harness {
    let mut harness = Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(100));
    harness
}

#[test]
fn a_list_opted_in_takes_the_keyboard_as_it_mounts_and_one_not_does_not() {
    assert!(harness(Mounted).focus_of(".ds-list"));
    assert!(!harness(Manual).focus_of(".ds-list"));
}
