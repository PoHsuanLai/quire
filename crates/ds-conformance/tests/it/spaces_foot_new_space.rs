//! The Space foot's New Space button on a real Blitz document: it is drawn by default and left
//! out by `NewSpace::Hidden`, which keeps the dots.

use dioxus::prelude::*;
use ds::components::app::spaces::{Spaces, SpacesFoot, Switched, use_spaces};
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Part, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 420,
    height: 200,
    scale_percent: 100,
};

fn foot(new_space: NewSpace) -> Element {
    let spaces = use_spaces(
        || Spaces::first_run((0..3).map(|_| ()), || ()),
        |_| {},
        String::new,
        |_: Switched<String>| {},
    );
    let new_payload = use_callback(|()| ());
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            div { style: "padding:24px; width:260px",
                SpacesFoot { handle: spaces, new_payload, new_space }
            }
        }
    }
}

#[allow(non_snake_case)]
fn WithPlus() -> Element {
    foot(NewSpace::Shown)
}

#[allow(non_snake_case)]
fn WithoutPlus() -> Element {
    foot(NewSpace::Hidden)
}

fn started(app: fn() -> Element) -> Harness {
    let mut harness = Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(100));
    harness
}

#[test]
fn the_foot_draws_its_plus_unless_asked_not_to() {
    let shown = started(WithPlus);
    assert_eq!(shown.count(".ds-spaces-foot > .ds-button"), 1);
    let hidden = started(WithoutPlus);
    assert_eq!(hidden.count(".ds-spaces-foot > .ds-button"), 0);
    assert_eq!(hidden.count(".ds-spaces-dot-hold"), 3, "the dots stay");
}

/// The foot draws each Space as the foot face: a 22px target around a small pip, the current
/// one a larger ink pip and the others a smaller, fainter one; the labels and pressed states
/// an app reads stay as they were.
#[test]
fn the_foot_dots_are_small_pips_and_the_current_one_is_larger_ink() {
    let mut harness = started(WithPlus);
    harness.advance(Duration::from_millis(500));
    let dot =
        |n: usize| format!(".ds-spaces-dots > .ds-spaces-dot-hold:nth-child({n}) .ds-space-dot");
    let pip = |n: usize| format!("{} > .ds-space-pip", dot(n));
    for n in 1..=3 {
        assert_eq!(harness.attr(&dot(n), "data-face").as_deref(), Some("foot"));
        assert_eq!(harness.attr(&dot(n), "data-stops"), None, "no colour fill");
        let label = harness.attr(&dot(n), "aria-label").expect("label");
        assert!(label.ends_with(" Space"), "label {label}");
        let target = harness.rect(&dot(n)).expect("target");
        assert!(
            (target.size.width.0 - 22.0).abs() < 1.0,
            "target {}",
            target.size.width.0
        );
    }
    assert_eq!(
        harness.attr(&dot(1), "aria-pressed").as_deref(),
        Some("true")
    );
    assert_eq!(
        harness.attr(&dot(2), "aria-pressed").as_deref(),
        Some("false")
    );
    let current = harness.rect(&pip(1)).expect("current pip").size.width.0;
    let other = harness.rect(&pip(2)).expect("other pip").size.width.0;
    assert!((current - 7.0).abs() < 0.5, "current pip {current}");
    assert!((other - 5.0).abs() < 0.5, "other pip {other}");
    let ink = harness.fill_of(&pip(1), Part::Element).expect("fill");
    let faint = harness.fill_of(&pip(2), Part::Element).expect("fill");
    assert!(ink != faint, "brightness marks the current one");
}
