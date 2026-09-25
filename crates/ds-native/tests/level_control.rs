//! The level control on a real Blitz document (the user's brief of 2026-09-25; sill FINDINGS
//! Q74): a level set from outside slides over `--t-quick --e-out`, while under the pointer the
//! fill follows it with no easing; a press swells the track; a drag past the end stretches the
//! capsule and it springs back on release (not under Reduced); keys step by sixteenths, Shift by
//! sixty-fourths. Measured on the painted pixels and the laid-out rects.

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::{
    Appearance, Ds, DurationToken, EasingToken, Fraction, Key, LevelControl, LevelGlyph, Material,
    Motion, MotionLevel, Muting, Point, Px, Theme,
};
use ds_native::{Harness, Viewport};
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
                LevelControl {
                    label: "Volume",
                    value: VALUE(),
                    glyph: LevelGlyph::Volume(Muting::Audible),
                    onchange: move |next| *VALUE.write() = next,
                }
            }
        }
    }
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn start(value: u16, motion: Motion) -> Harness {
    MOTION.set(motion);
    let mut harness = Harness::new(Level, VIEW);
    harness.within(|| *VALUE.write() = Fraction(value));
    harness.advance(ms(400));
    harness
}

/// The painted fill's width: how far the fill's colour runs along the track's middle row from
/// its left end, past the glyph.
fn painted(harness: &mut Harness, name: &str) -> u32 {
    let track = rect(harness, ".ds-level-track");
    let frame = harness.render().expect("renders");
    keep(&frame, name);
    let row = (track.origin.y.0 + 20.0) as u32;
    run_from(&frame, track.origin.x.0 as u32, row)
}

fn run_from(frame: &RgbaImage, left: u32, row: u32) -> u32 {
    let fill = frame.get_pixel(left + 3, row).0;
    (left + 3..frame.width())
        .take_while(|&x| distance(frame.get_pixel(x, row).0, fill) <= 16)
        .count() as u32
        + 3
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
    let quarter = width(&harness, ".ds-level-fill");
    harness.advance(quick / 4);
    let half = width(&harness, ".ds-level-fill");
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
    let rail = rect(&harness, ".ds-level-rail");
    let at = |share: f32| Point {
        x: Px(rail.origin.x.0 + rail.size.width.0 * share),
        y: Px(rail.origin.y.0 + 13.0),
    };
    harness.pointer_down(at(0.3));
    // The track is measured once per press, after layout; then the fill is the pointer's.
    harness.advance(ms(80));
    let pressed = width(&harness, ".ds-level-fill");
    harness.pointer_move(at(0.7));
    harness.advance(ms(16));
    let moved = width(&harness, ".ds-level-fill");
    let followed = painted(&mut harness, "level-drag");
    println!("pressed at 30 %: {pressed:.1}; moved to 70 %: {moved:.1} (painted {followed})");
    assert_eq!(level(&mut harness), 700);
    assert_eq!(
        harness.attr(".ds-level", "data-drag").as_deref(),
        Some("live")
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

/// Painted rows of the fill's colour down the column at `x`: the fill's painted height.
fn painted_height(frame: &RgbaImage, x: u32, around: u32) -> u32 {
    let fill = frame.get_pixel(x, around).0;
    (around.saturating_sub(24)..around + 24)
        .filter(|&y| distance(frame.get_pixel(x, y).0, fill) <= 2)
        .count() as u32
}

#[test]
fn a_press_swells_and_a_drag_past_the_end_stretches_then_springs_back() {
    let mut harness = start(500, Motion::Standard);
    let rail = rect(&harness, ".ds-level-rail");
    let middle = (rail.origin.y.0 + 13.0) as u32;
    // On the fill (the press lands at 50 %), past the glyph: white against the card.
    let probe_x = (rail.origin.x.0 + 60.0) as u32;
    let rest = painted_height(&harness.render().expect("renders"), probe_x, middle);
    let at = |x: f32| Point {
        x: Px(x),
        y: Px(rail.origin.y.0 + 13.0),
    };
    harness.pointer_down(at(rail.origin.x.0 + 100.0));
    harness.advance(ms(300));
    let swollen = painted_height(&harness.render().expect("renders"), probe_x, middle);
    let right = rail.origin.x.0 + rail.size.width.0;
    harness.pointer_move(at(right + 12.0));
    harness.advance(ms(16));
    let stretched = width(&harness, ".ds-level-track");
    keep(&harness.render().expect("renders"), "level-stretched");
    harness.pointer_up(at(right + 12.0));
    harness.advance(ms(600));
    let released = width(&harness, ".ds-level-track");
    println!(
        "height at rest {rest}, pressed {swollen}; width {} -> stretched {stretched:.1} -> released {released:.1}",
        rail.size.width.0
    );
    // scaleY(1.08) on 26 px: a pixel above and below.
    assert!(
        swollen >= rest + 2,
        "the press swells the track: {rest} -> {swollen}"
    );
    // 12 px past the end: 6 x 12 / (12 + 12) = 3 px.
    assert!(
        (stretched - rail.size.width.0 - 3.0).abs() <= 0.5,
        "{stretched}"
    );
    assert!(
        (released - rail.size.width.0).abs() <= 0.5,
        "springs back: {released}"
    );
    assert_eq!(level(&mut harness), 1000, "the level stays at the end");
}

#[test]
fn reduced_motion_has_no_rubber_band() {
    let mut harness = start(500, Motion::Reduced);
    let rail = rect(&harness, ".ds-level-rail");
    let y = Px(rail.origin.y.0 + 13.0);
    harness.pointer_down(Point {
        x: Px(rail.origin.x.0 + 100.0),
        y,
    });
    harness.advance(ms(16));
    harness.pointer_move(Point {
        x: Px(rail.origin.x.0 + rail.size.width.0 + 12.0),
        y,
    });
    harness.advance(ms(16));
    assert_eq!(width(&harness, ".ds-level-track"), rail.size.width.0);
    assert_eq!(harness.attr(".ds-level", "data-over"), None);
}

#[test]
fn keys_step_by_sixteenths_and_shift_by_sixty_fourths() {
    let mut harness = start(500, Motion::Standard);
    let rail = rect(&harness, ".ds-level-rail");
    let at = Point {
        x: Px(rail.origin.x.0 + 100.0),
        y: Px(rail.origin.y.0 + 13.0),
    };
    harness.click(at);
    harness.advance(ms(120));
    assert_eq!(level(&mut harness), 500, "a click at the middle keeps 50 %");
    assert!(
        harness.is_focused(".ds-level"),
        "the press focuses the control"
    );
    harness.key(Key::Right);
    assert_eq!(level(&mut harness), 563);
    harness.key(Key::Left);
    harness.key(Key::Left);
    assert_eq!(level(&mut harness), 438);
    harness.chord(&[Key::Shift], Key::Up);
    assert_eq!(level(&mut harness), 453);
}
