//! The level control on a real Blitz document (the user's brief of 2026-09-25):
//! a level set from outside slides over `--t-quick --e-out`, while under the pointer the
//! fill follows it with no easing; a drag past the end never stretches the track; keys step by
//! sixteenths, Shift by sixty-fourths. Measured on the painted pixels and the laid-out rects.

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::base::vocab::Muting;
use ds::components::content::level_glyph::vocab::LevelGlyph;
use ds::components::controls::slider_model::SliderLook;
use ds::prelude::*;
use ds::style::tokens::easing::EasingToken;
use ds::style::tokens::timing::DurationToken;
use ds_harness::{Clock, Driver, FocusState, Harness, HarnessConfig, Input, Query, Viewport};
use image::RgbaImage;
use probe::{distance, keep, rect};
use std::cell::Cell;
use std::time::Duration;

static VALUE: GlobalSignal<Fraction> = Signal::global(|| Fraction(200));

thread_local! {
    static MOTION: Cell<Motion> = const { Cell::new(Motion::Standard) };
}

const VIEW: Viewport = Viewport {
    width: 320,
    height: 90,
    scale_percent: 100,
};

/// The rail is 200 px wide from x 60 (room for the rubber band either side).
const PROBE_CSS: &str = ".bar{margin:30px 0 0 60px;width:200px}";

#[allow(non_snake_case)]
fn Level() -> Element {
    rsx! {
        Ds {
            appearance: Appearance { theme: Theme::Light, motion: MOTION.get(), ..Appearance::default() },
            material: Material::Osd,
            style { {PROBE_CSS} }
            div { class: "bar",
                Slider {
                    label: "Volume",
                    value: VALUE(),
                    glyph: LevelGlyph::Volume(Muting::Audible),
                    onchange: move |next| *VALUE.write() = next, look: SliderLook::Capsule }
            }
        }
    }
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn start(value: u16, motion: Motion) -> Harness {
    MOTION.set(motion);
    let mut harness = Harness::new(Level, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.within(|| *VALUE.write() = Fraction(value));
    harness.advance(ms(400));
    harness
}

/// The painted fill's width: how far the fill's colour runs along the track's middle row from
/// its left end, past the glyph.
fn painted(harness: &mut Harness, name: &str) -> u32 {
    let track = rect(harness, ".ds-slider-track");
    let frame = harness.render().expect("renders");
    keep(&frame, name);
    // Below the glyph's ink (the glyph is centred in the capsule: 16 px in the 28 Regular one),
    // 5 px above the capsule's bottom edge, where the round end cuts a few px off the run.
    let row = (track.origin.y.0 + track.size.height.0 - 5.0) as u32;
    run_from(&frame, track.origin.x.0 as u32, row)
}

fn run_from(frame: &RgbaImage, left: u32, row: u32) -> u32 {
    let fill = frame.get_pixel(left + 6, row).0;
    (left + 6..frame.width())
        .take_while(|&x| distance(frame.get_pixel(x, row).0, fill) <= 16)
        .count() as u32
        + 6
}

/// The level the control last reported, read inside the app's runtime.
fn level(harness: &mut Harness) -> u16 {
    harness.within(|| VALUE.peek().0)
}

fn width(harness: &Harness, selector: &str) -> f32 {
    rect(harness, selector).size.width.0
}

/// The fill's width `share` thousandths into `--t-quick` on the `--e-out` curve, from 20 % to
/// 80 % of 200 px.
fn on_the_curve(share: u16) -> f32 {
    let curve = EasingToken::Out.easing(MotionLevel::Standard);
    let done = f32::from(curve.at(Fraction(share)).0) / 1000.0;
    200.0 * (0.2 + 0.6 * done)
}

#[test]
fn a_level_set_from_outside_slides_over_t_quick() {
    let mut harness = start(200, Motion::Standard);
    let before = painted(&mut harness, "level-set-before");
    harness.within(|| *VALUE.write() = Fraction(800));
    let quick = DurationToken::Quick.duration(MotionLevel::Standard);
    harness.advance(quick / 4);
    let quarter = width(&harness, ".ds-slider-fill");
    harness.advance(quick / 4);
    let half = width(&harness, ".ds-slider-fill");
    let half_painted = painted(&mut harness, "level-set-half");
    harness.advance(quick * 2);
    let after = painted(&mut harness, "level-set-after");
    println!(
        "painted {before} -> {half_painted} -> {after}; laid out a quarter {quarter:.1} (curve {:.1}), half {half:.1} (curve {:.1})",
        on_the_curve(250),
        on_the_curve(500)
    );
    // The painted run ends a pixel or so short of the laid-out edge, where the round end turns.
    assert!(before.abs_diff(40) <= 4, "20 % of 200 px: {before}");
    assert!(after.abs_diff(160) <= 4, "80 % of 200 px: {after}");
    assert!(
        before + 10 < half_painted && half_painted + 3 < after,
        "the fill jumped: {before} -> {half_painted} -> {after}"
    );
    for (got, want) in [(quarter, on_the_curve(250)), (half, on_the_curve(500))] {
        assert!(
            (got - want).abs() <= 3.0,
            "{got:.1} is not the curve's {want:.1}"
        );
    }
}

#[test]
fn under_the_pointer_the_fill_follows_with_no_easing() {
    let mut harness = start(200, Motion::Standard);
    let rail = rect(&harness, ".ds-slider-rail");
    let at = |share: f32| Point {
        x: Px(rail.origin.x.0 + rail.size.width.0 * share),
        y: Px(rail.origin.y.0 + 11.0),
    };
    harness.send(Input::pointer_down(at(0.3)));
    // The track is measured once per press, after layout; then the fill is the pointer's.
    harness.advance(ms(80));
    let pressed = width(&harness, ".ds-slider-fill");
    harness.send(Input::pointer_move(at(0.7)));
    harness.advance(ms(16));
    let moved = width(&harness, ".ds-slider-fill");
    let followed = painted(&mut harness, "level-drag");
    println!("pressed at 30 %: {pressed:.1}; moved to 70 %: {moved:.1} (painted {followed})");
    assert_eq!(level(&mut harness), 700);
    assert_eq!(
        harness.attr(".ds-slider", "data-pressed").as_deref(),
        Some("true")
    );
    assert!(
        (pressed - 60.0).abs() <= 1.0,
        "the press jumps to the pointer: {pressed}"
    );
    assert!(
        (moved - 140.0).abs() <= 1.0,
        "16 ms after the move the fill is where the pointer is: {moved}"
    );
    assert!(followed.abs_diff(140) <= 4, "painted {followed}");
}

#[test]
fn a_drag_past_the_end_never_stretches_the_track() {
    let mut harness = start(500, Motion::Standard);
    let rail = rect(&harness, ".ds-slider-rail");
    let y = Px(rail.origin.y.0 + 13.0);
    harness.send(Input::pointer_down(Point {
        x: Px(rail.origin.x.0 + 100.0),
        y,
    }));
    harness.advance(ms(16));
    harness.send(Input::pointer_move(Point {
        x: Px(rail.origin.x.0 + rail.size.width.0 + 12.0),
        y,
    }));
    harness.advance(ms(16));
    assert_eq!(width(&harness, ".ds-slider-track"), rail.size.width.0);
    assert_eq!(harness.attr(".ds-slider", "data-over"), None);
}

#[test]
fn keys_step_by_sixteenths_and_shift_by_sixty_fourths() {
    let mut harness = start(500, Motion::Standard);
    let rail = rect(&harness, ".ds-slider-rail");
    let at = Point {
        x: Px(rail.origin.x.0 + 100.0),
        y: Px(rail.origin.y.0 + 11.0),
    };
    harness.send(Input::click(at));
    harness.advance(ms(120));
    assert_eq!(level(&mut harness), 500, "a click at the middle keeps 50 %");
    assert_eq!(
        harness.focus_of(".ds-slider"),
        FocusState::Focused,
        "the press focuses the control"
    );
    harness.send(Input::key(ShortcutKey::Right));
    assert_eq!(level(&mut harness), 563);
    harness.send(Input::key(ShortcutKey::Left));
    harness.send(Input::key(ShortcutKey::Left));
    assert_eq!(level(&mut harness), 438);
    harness.send(Input::chord(&[ShortcutKey::Shift], ShortcutKey::Up));
    assert_eq!(level(&mut harness), 453);
}
