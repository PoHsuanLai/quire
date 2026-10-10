//! The Space foot's New Space button on a real Blitz document: it is drawn by default and left
//! out by `NewSpace::Hidden`, which keeps the dots.

use dioxus::prelude::*;
use ds::components::app::spaces::{Spaces, SpacesFoot, Switched, use_spaces};
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Query, Viewport};
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
