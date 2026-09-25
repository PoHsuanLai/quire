//! sill Q182 on a real Blitz document: a widget's value bumps once when it changes, and is at
//! rest again once the bump has settled. Nothing plays on mount and nothing loops.

use dioxus::prelude::*;
use ds::{
    Anim, Appearance, Bumped, Button, ButtonVariant, Ds, Fraction, LevelRing, Material,
    MotionLevel, RootChrome, StaggerIndex, WidgetFrame, WidgetMetrics, WidgetSize, settle,
};
use ds_native::harness::settle_until;
use ds_native::{Harness, Viewport};
use std::time::{Duration, Instant};

const VIEW: Viewport = Viewport {
    width: 480,
    height: 320,
    scale_percent: 100,
};

/// A battery widget whose level a button lowers by a tenth.
#[allow(non_snake_case)]
fn Battery() -> Element {
    let mut level = use_signal(|| Fraction(800));
    let percent = level().0 / 10;
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Widget, chrome: Some(RootChrome::Transparent),
            div { style: WidgetMetrics::default().style_attr(),
                Button { id: "drain", variant: ButtonVariant::Mini, label: "Drain",
                    onclick: move |_| level.set(Fraction(level().0.saturating_sub(100))) }
                WidgetFrame { size: WidgetSize::Small,
                    LevelRing { level: level(), label: "This computer" }
                    Bumped { on: percent, span { class: "ds-count", "{percent}%" } }
                }
            }
        }
    }
}

fn ring_bumping(harness: &Harness) -> bool {
    harness.has_class(".ds-ring", "a-bump")
}

fn text_bumping(harness: &Harness) -> bool {
    harness.has_class(".ds-bumped", "a-bump")
}

/// One change: the ring and the text each bump, on the first alias, and both are at rest again
/// no sooner than `settle(Bump)` after the change, and stay at rest.
#[test]
fn a_change_bumps_once_and_settles() {
    let mut harness = Harness::new(Battery, VIEW);
    harness.advance(Duration::from_millis(100));
    assert!(!ring_bumping(&harness), "nothing bumps on mount");
    assert!(!text_bumping(&harness), "nothing bumps on mount");

    let changed = Instant::now();
    harness.click(harness.centre("#drain").expect("the drain button"));
    settle_until(&mut harness, |h| ring_bumping(h) && text_bumping(h));
    assert_eq!(harness.attr(".ds-ring", "data-pulse").as_deref(), Some("a"));
    assert_eq!(
        harness.attr(".ds-ring", "aria-valuenow").as_deref(),
        Some("70")
    );

    let settles = settle(Anim::Bump, MotionLevel::Standard, StaggerIndex::new(0));
    let rested = settle_until(&mut harness, |h| !ring_bumping(h) && !text_bumping(h));
    assert!(
        rested.duration_since(changed) >= settles,
        "at rest only once the whole bump had run: {:?}",
        rested.duration_since(changed)
    );
    assert_eq!(harness.attr(".ds-ring", "data-pulse"), None);

    // Nothing loops: a further settle's worth of time brings no second bump.
    harness.advance(settles);
    assert!(!ring_bumping(&harness), "{}", harness.html());
    assert!(!text_bumping(&harness), "{}", harness.html());
}

/// A second change bumps again, on the other alias, so the keyframe restarts.
#[test]
fn a_second_change_bumps_again_on_the_other_alias() {
    let mut harness = Harness::new(Battery, VIEW);
    harness.advance(Duration::from_millis(100));
    harness.click(harness.centre("#drain").expect("drain"));
    settle_until(&mut harness, ring_bumping);
    settle_until(&mut harness, |h| !ring_bumping(h));
    harness.click(harness.centre("#drain").expect("drain"));
    settle_until(&mut harness, ring_bumping);
    assert_eq!(harness.attr(".ds-ring", "data-pulse").as_deref(), Some("b"));
    assert_eq!(
        harness.attr(".ds-ring", "aria-valuenow").as_deref(),
        Some("60")
    );
    settle_until(&mut harness, |h| !ring_bumping(h));
}
