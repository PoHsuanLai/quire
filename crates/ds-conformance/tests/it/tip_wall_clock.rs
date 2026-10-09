//! A tip over a button on the wall clock (real `futures-timer` sleeps, as in a window), at the
//! scales a desktop runs at, and placed below its button, not only shown on the virtual clock at
//! 100%. (The tip that never showed in a real window was a missing anchor: see
//! `ds-blitz/tests/it/provide_host.rs`; the harness wires its host on the root, a window did not.)
use dioxus::prelude::*;
use ds::components::controls::button_model::{Bezel, ImagePosition};
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            div { style: "position:absolute;left:100px;top:100px;display:flex;gap:8px",
                Button {
                    label: "first",
                    title: Some("Archive".to_owned()),
                    bezel: Bezel::Toolbar,
                    size: ControlSize::Large,
                    image: ImagePosition::Only,
                    icon: Some(IconSource::from(Icon::Archive)),
                    onclick: |_| {},
                    common: Common { id: Some("first".to_owned()), ..Common::default() },
                }
            }
        }
    }
}

fn rests_on_the_button(clock: Clock, scale_percent: u16) -> usize {
    let view = Viewport {
        width: 640,
        height: 400,
        scale_percent,
    };
    let mut harness = Harness::new(Page, HarnessConfig::new(view).with_clock(clock));
    harness.advance(Duration::from_millis(200));
    let first = harness.centre("#first").expect("first");
    harness.send(Input::pointer_move(first));
    harness.advance(Duration::from_millis(1400));
    harness.count(".ds-tooltip")
}

#[test]
fn a_tip_shows_on_the_wall_clock_at_every_scale() {
    for scale in [100, 150, 200] {
        assert_eq!(
            rests_on_the_button(Clock::Wall, scale),
            1,
            "wall clock, {scale}%"
        );
    }
}

#[test]
fn a_tip_shows_on_the_virtual_clock_at_every_scale() {
    for scale in [100, 150, 200] {
        assert_eq!(
            rests_on_the_button(Clock::Virtual, scale),
            1,
            "virtual clock, {scale}%"
        );
    }
}

#[test]
fn a_tip_stands_below_its_button_at_every_scale() {
    for scale in [100, 150, 200] {
        let view = Viewport {
            width: 640,
            height: 400,
            scale_percent: scale,
        };
        let mut harness = Harness::new(Page, HarnessConfig::new(view).with_clock(Clock::Wall));
        harness.advance(Duration::from_millis(200));
        let first = harness.centre("#first").expect("first");
        harness.send(Input::pointer_move(first));
        harness.advance(Duration::from_millis(1400));
        let tip = harness.centre(".ds-tooltip").expect("tip");
        assert!(
            tip.y.0 > first.y.0 + 20.0 && (tip.x.0 - first.x.0).abs() < 2.0,
            "{scale}%: tip at {tip:?} is not centred below the button at {first:?}"
        );
    }
}
