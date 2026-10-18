//! The Space foot on a real Blitz document: its New Space button is drawn by default and left
//! out by `NewSpace::Hidden`, which keeps the dots; with one Space it draws no dots at all.

use dioxus::prelude::*;
use ds::components::app::spaces::{Spaces, SpacesFoot, Switched, use_spaces};
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Part, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 420,
    height: 200,
    scale_percent: 100,
};

fn foot(new_space: NewSpace, count: usize) -> Element {
    let spaces = use_spaces(
        move || Spaces::first_run((0..count).map(|_| ()), || ()),
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
    foot(NewSpace::Shown, 3)
}

#[allow(non_snake_case)]
fn WithoutPlus() -> Element {
    foot(NewSpace::Hidden, 3)
}

#[allow(non_snake_case)]
fn OneSpace() -> Element {
    foot(NewSpace::Shown, 1)
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

/// One Space has nothing to switch to: the foot draws no dot and no dots group, and the New
/// Space button still sits at the foot's end.
#[test]
fn one_space_draws_no_dots_and_no_dots_group() {
    let many = started(WithPlus);
    assert_eq!(many.count(".ds-spaces-dots"), 1);
    assert_eq!(many.count(".ds-space-dot"), 3);
    let one = started(OneSpace);
    assert_eq!(one.count(".ds-spaces-dots"), 0, "no dots group");
    assert_eq!(one.count(".ds-space-dot"), 0, "no dot");
    assert_eq!(one.count(".ds-spaces-foot [role=group]"), 0);
    let foot = one.rect(".ds-spaces-foot").expect("foot");
    let plus = one.rect(".ds-spaces-foot > .ds-button").expect("plus");
    let foot_end = foot.origin.x.0 + foot.size.width.0;
    let plus_end = plus.origin.x.0 + plus.size.width.0;
    assert!(
        (foot_end - plus_end).abs() < 1.0,
        "New Space ends at {plus_end}, the foot at {foot_end}"
    );
}

/// The foot draws each Space as the foot face: a 16px target around a small pip, the current
/// one a larger ink pip and the others a smaller, fainter one, the targets 2px apart and
/// together at the centre of the dots' width; the labels and pressed states an app reads stay
/// as they were.
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
            (target.size.width.0 - 16.0).abs() < 1.0,
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
    assert!((current - 6.0).abs() < 0.5, "current pip {current}");
    assert!((other - 4.0).abs() < 0.5, "other pip {other}");
    let first = harness.rect(&dot(1)).expect("first");
    let second = harness.rect(&dot(2)).expect("second");
    let last = harness.rect(&dot(3)).expect("last");
    let apart = second.origin.x.0 - (first.origin.x.0 + first.size.width.0);
    assert!((apart - 2.0).abs() < 0.5, "dots {apart}px apart");
    let group = harness.rect(".ds-spaces-dots").expect("dots");
    let middle = (first.origin.x.0 + last.origin.x.0 + last.size.width.0) / 2.0;
    let centre = group.origin.x.0 + group.size.width.0 / 2.0;
    assert!(
        (middle - centre).abs() < 1.0,
        "dots at {middle}, their width's centre {centre}"
    );
    let ink = harness.fill_of(&pip(1), Part::Element).expect("fill");
    let faint = harness.fill_of(&pip(2), Part::Element).expect("fill");
    assert!(ink != faint, "brightness marks the current one");
}
