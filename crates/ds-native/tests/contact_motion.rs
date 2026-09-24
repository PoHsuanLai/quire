//! The contact keyframes (`gulp`, `bump`, `seal-pop`) follow the motion level like `pop-in`
//! does (mailo gaps 3, item 4): their squash and overshoot scale by `--overshoot`, so Calm
//! (and Reduced), where `--overshoot` is 1, flattens them to no change of shape, while Standard
//! and Extra keep the catalogue's shape. Measured on the painted pixels of a 100 px ink square
//! at the keyframe's own offset, where the animation sits exactly on that keyframe.

use dioxus::prelude::*;
use ds::{Anim, Appearance, Ds, Material, Motion, MotionLevel, PulseKey};
use ds_native::{Harness, Viewport};
use std::cell::Cell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
    scale_percent: 100,
};

thread_local! {
    static CASE: Cell<(Anim, Motion)> = const { Cell::new((Anim::Gulp, Motion::Standard)) };
}

/// A 100 px ink square playing the case's pulse from its first frame.
#[allow(non_snake_case)]
fn Square() -> Element {
    let (anim, motion) = CASE.get();
    let (class, alias) = PulseKey::rest(anim)
        .fired()
        .attrs()
        .expect("a fired pulse plays");
    rsx! {
        Ds {
            appearance: Appearance { motion, ..Appearance::default() },
            material: Material::Popover,
            div {
                class: "{class}",
                "data-pulse": alias,
                style: "position:absolute;left:150px;top:150px;width:100px;height:100px;background:var(--ink)",
            }
        }
    }
}

/// The painted square's width and height, in pixels, `at` into the animation.
fn painted(anim: Anim, motion: Motion, at: Duration) -> (u32, u32) {
    CASE.set((anim, motion));
    let mut harness = Harness::new(Square, VIEW);
    harness.advance(at);
    let frame = harness.render().expect("renders");
    let ink: Vec<(u32, u32)> = frame
        .enumerate_pixels()
        .filter(|(_, _, pixel)| pixel.0[..3].iter().all(|channel| *channel < 90))
        .map(|(x, y, _)| (x, y))
        .collect();
    let span = |axis: fn(&(u32, u32)) -> u32| {
        let (low, high) = (ink.iter().map(axis).min(), ink.iter().map(axis).max());
        low.zip(high).map_or(0, |(low, high)| high - low + 1)
    };
    (span(|p| p.0), span(|p| p.1))
}

/// Where the keyframe's offset falls at `level`: `offset` thousandths of the recipe's
/// duration, which is exactly that keyframe whatever the easing (it applies between frames).
fn at_offset(anim: Anim, level: MotionLevel, offset: u32) -> Duration {
    anim.recipe().duration.duration(level) * offset / 1000
}

fn close(got: u32, want: u32) -> bool {
    got.abs_diff(want) <= 2
}

#[test]
fn calm_flattens_the_contact_keyframes_and_standard_keeps_their_shape() {
    // (anim, the keyframe's offset, Standard's size there): gulp 34% scale(1.07, .84),
    // bump 40% scale(1.25), seal-pop 60% scale(1.5).
    const CASES: &[(Anim, u32, (u32, u32))] = &[
        (Anim::Gulp, 340, (107, 84)),
        (Anim::Bump, 400, (125, 125)),
        (Anim::SealPop, 600, (150, 150)),
    ];
    let mut failures = Vec::new();
    for &(anim, offset, standard) in CASES {
        let got = painted(
            anim,
            Motion::Standard,
            at_offset(anim, MotionLevel::Standard, offset),
        );
        if !(close(got.0, standard.0) && close(got.1, standard.1)) {
            failures.push(format!("{anim:?} Standard: {got:?}, want {standard:?}"));
        }
        let calm = painted(
            anim,
            Motion::Calm,
            at_offset(anim, MotionLevel::Calm, offset),
        );
        if !(close(calm.0, 100) && close(calm.1, 100)) {
            failures.push(format!("{anim:?} Calm: {calm:?}, want (100, 100)"));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn extra_keeps_the_standard_shape() {
    // The amplitude is capped at Standard's: Extra's own spring curve already overshoots more.
    let got = painted(
        Anim::Bump,
        Motion::Extra,
        at_offset(Anim::Bump, MotionLevel::Extra, 400),
    );
    assert!(close(got.0, 125) && close(got.1, 125), "{got:?}");
}
