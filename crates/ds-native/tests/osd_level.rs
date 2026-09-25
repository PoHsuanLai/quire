//! The read-only level bar slides (sill FINDINGS Q74; design/20 section 1.7, "level change
//! `--t-quick` `--e-out`"): after the level changes from 20 % to 80 %, a quarter and a half of
//! `--t-quick` in, the painted fill ends between the two levels, each time where the `--e-out`
//! curve puts it (at half, `--e-out` is already about nine tenths of the way); once the
//! transition has settled it ends at 80 %. Measured on the painted accent pixels of the track.

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::{
    Appearance, Button, ButtonVariant, Ds, DurationToken, EasingToken, Fraction, Material,
    MotionLevel, Slider, SliderMode, Theme,
};
use ds_native::{Harness, Viewport};
use image::RgbaImage;
use probe::{distance, keep};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 320,
    height: 120,
    scale_percent: 100,
};

/// The track is 200 px wide from x 20; the button sits below it, out of the probed row.
const WIDTH: f32 = 200.0;
const LEFT: u32 = 20;
const PROBE_CSS: &str = ".bar{margin:20px 0 0 20px;width:200px}.go{margin:20px 0 0 20px}";

#[allow(non_snake_case)]
fn Level() -> Element {
    let mut value = use_signal(|| Fraction(200));
    rsx! {
        Ds {
            appearance: Appearance { theme: Theme::Light, ..Appearance::default() },
            material: Material::Window,
            style { {PROBE_CSS} }
            div { class: "bar",
                Slider { label: "Volume", value: value(), mode: SliderMode::Level }
            }
            div { class: "go",
                Button { variant: ButtonVariant::Quiet, label: "Louder", onclick: move |_| value.set(Fraction(800)) }
            }
        }
    }
}

/// The painted fill's width: from the track's left edge, how far the fill's own colour runs
/// along the track's middle row.
fn filled(frame: &RgbaImage, row: u32) -> u32 {
    let fill = frame.get_pixel(LEFT + 3, row).0;
    let run = (LEFT + 3..LEFT + WIDTH as u32)
        .take_while(|&x| distance(frame.get_pixel(x, row).0, fill) <= 24)
        .count() as u32;
    run + 3
}

/// Where the `--e-out` curve is `share` thousandths into `--t-quick`, as a fill width between
/// 20 % and 80 %.
fn expected(share: u16) -> f32 {
    let curve = EasingToken::Out.easing(MotionLevel::Standard);
    let done = f32::from(curve.at(Fraction(share)).0) / 1000.0;
    WIDTH * (0.2 + 0.6 * done)
}

#[test]
fn a_level_change_slides_the_fill_over_t_quick() {
    let mut harness = Harness::new(Level, VIEW);
    harness.advance(Duration::from_millis(60));
    let track = probe::rect(&harness, ".ds-slider-track");
    let row = (track.origin.y.0 + track.size.height.0 / 2.0) as u32;
    let before = filled(&harness.render().expect("renders"), row);
    assert_eq!(
        harness.count(".ds-slider-thumb"),
        0,
        "a level bar draws no thumb"
    );
    let go = harness
        .centre(".go .ds-button")
        .expect("the button is drawn");
    harness.click(go);
    let quick = DurationToken::Quick.duration(MotionLevel::Standard);
    harness.advance(quick / 4);
    let quarter = filled(&harness.render().expect("renders"), row);
    harness.advance(quick / 4);
    let halfway = harness.render().expect("renders");
    keep(&halfway, "osd-level-halfway");
    let middle = filled(&halfway, row);
    harness.advance(quick * 2);
    let settled = harness.render().expect("renders");
    keep(&settled, "osd-level-settled");
    let after = filled(&settled, row);
    let (want_quarter, want_middle) = (expected(250), expected(500));
    println!(
        "before {before} px, a quarter {quarter} px (curve {want_quarter:.1}), half {middle} px \
         (curve {want_middle:.1}), after {after} px"
    );
    assert!(before.abs_diff(40) <= 2, "20 % of 200 px: {before}");
    assert!(after.abs_diff(160) <= 2, "80 % of 200 px: {after}");
    assert!(
        before + 10 < quarter && quarter < middle && middle + 3 < after,
        "the fill jumped instead of sliding: {before} -> {quarter} -> {middle} -> {after}"
    );
    for (got, want, when) in [
        (quarter, want_quarter, "a quarter"),
        (middle, want_middle, "half"),
    ] {
        assert!(
            (got as f32 - want).abs() <= 3.0,
            "{when} of --t-quick the fill is {got} px, the --e-out curve says {want:.1}"
        );
    }
}
