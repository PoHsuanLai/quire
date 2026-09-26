//! design/25-EMOJI.md section 5 on a real Blitz document: an animated emoji's frame advances
//! while it is awake and, 21 s after the wake, rests on frame 0 with nothing scheduled or
//! painting (the idle-frame rule); a wince shows the wrong-password emoji, then the user's
//! own again; under Reduced motion, or with `EmojiPlayback::Still`, the frame never leaves 0.

use dioxus::prelude::*;
use ds::{
    AnimatedEmoji, Appearance, Ds, EmojiId, EmojiPlayback, Material, Mood, Motion, PersonaSize,
    Theme,
};
use ds_native::harness::settle_until;
use ds_native::{Harness, Viewport};
use std::time::{Duration, Instant};

const VIEW: Viewport = Viewport {
    width: 200,
    height: 200,
    scale_percent: 100,
};

static MOOD: GlobalSignal<Mood> = Signal::global(|| Mood::Idle);
static MOTION: GlobalSignal<Motion> = Signal::global(|| Motion::Standard);

#[allow(non_snake_case)]
fn Stage() -> Element {
    rsx! {
        Ds { appearance: Appearance { theme: Theme::Light, motion: MOTION(), ..Appearance::default() }, material: Material::Window,
            AnimatedEmoji { emoji: EmojiId::Wink, size: PersonaSize::Large, mood: MOOD() }
        }
    }
}

fn frame(harness: &Harness) -> Option<String> {
    harness.attr(".ds-emoji-face", "data-frame")
}

fn face(harness: &Harness) -> Option<String> {
    harness.attr(".ds-emoji-face", "data-emoji")
}

/// Pixels of the render that are not the window's plain ground: the emoji's own paint.
fn inked(image: &image::RgbaImage) -> usize {
    let ground = *image.get_pixel(2, 2);
    image.pixels().filter(|pixel| **pixel != ground).count()
}

#[test]
fn the_frame_advances_while_awake_and_rests_twenty_one_seconds_after_the_wake() {
    let woke = Instant::now();
    let mut harness = Harness::new(Stage, VIEW);
    let first = frame(&harness);
    let before = harness.render().expect("render");
    let moved = settle_until(&mut harness, |h| frame(h) != first);
    assert!(
        moved.duration_since(woke) < Duration::from_secs(2),
        "the first frame advanced only after {:?}",
        moved.duration_since(woke)
    );
    let after = harness.render().expect("render");
    assert!(inked(&before) > 2000, "the emoji is not painted");
    assert_ne!(
        before.as_raw(),
        after.as_raw(),
        "the painted frame did not change"
    );
    // Several distinct frames while awake, not one flicker.
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..20 {
        harness.advance(Duration::from_millis(50));
        seen.extend(frame(&harness));
    }
    assert!(seen.len() >= 5, "only frames {seen:?} in a second");
    // Still awake at 9 s, under half the window: the loop is still playing.
    harness.advance(Duration::from_secs(9).saturating_sub(woke.elapsed()));
    let mut playing = std::collections::BTreeSet::new();
    for _ in 0..10 {
        harness.advance(Duration::from_millis(50));
        playing.extend(frame(&harness));
    }
    assert!(playing.len() >= 3, "stopped before the window: {playing:?}");
    // The window is 20 s; at 21 s it rests on frame 0 and nothing moves or paints.
    harness.advance(Duration::from_secs(21).saturating_sub(woke.elapsed()));
    assert_eq!(frame(&harness).as_deref(), Some("0"), "not at rest at 21 s");
    assert_eq!(face(&harness).as_deref(), Some("wink"));
    assert!(!harness.is_animating(), "asks for frames at rest");
    for _ in 0..10 {
        harness.advance(Duration::from_millis(100));
        assert_eq!(
            frame(&harness).as_deref(),
            Some("0"),
            "moved after the window"
        );
    }
    assert!(!harness.is_animating());
}

#[test]
fn a_wince_shows_the_reaction_then_the_users_own_emoji() {
    let mut harness = Harness::new(Stage, VIEW);
    harness.advance(Duration::from_millis(200));
    assert_eq!(
        face(&harness).as_deref(),
        Some("wink"),
        "a reaction on mount"
    );
    let asked = Instant::now();
    harness.within(|| *MOOD.write() = Mood::Wince);
    let swapped = settle_until(&mut harness, |h| face(h).as_deref() == Some("confounded"));
    let reaction_frames = settle_until(&mut harness, |h| {
        face(h).as_deref() == Some("confounded") && frame(h).as_deref() != Some("0")
    });
    assert!(reaction_frames >= swapped, "the reaction plays");
    let back = settle_until(&mut harness, |h| face(h).as_deref() == Some("wink"));
    // The reaction's loop is 1840 ms (manifest.json); it is shown once through, not cut short.
    assert!(
        back.duration_since(asked) >= Duration::from_millis(1800),
        "back after only {:?}",
        back.duration_since(asked)
    );
    assert_eq!(
        harness.attr(".ds-emoji", "data-mood").as_deref(),
        Some("wince")
    );
    // Once: at least a loop later it has not come back.
    for _ in 0..20 {
        harness.advance(Duration::from_millis(100));
        assert_eq!(
            face(&harness).as_deref(),
            Some("wink"),
            "the reaction played again"
        );
    }
}

#[test]
fn under_reduced_motion_only_still_frames_show() {
    let mut harness = Harness::new(Stage, VIEW);
    harness.within(|| *MOTION.write() = Motion::Reduced);
    harness.advance(Duration::from_millis(100));
    harness.within(|| *MOOD.write() = Mood::Attentive);
    for _ in 0..10 {
        harness.advance(Duration::from_millis(100));
        assert_eq!(
            frame(&harness).as_deref(),
            Some("0"),
            "a frame under Reduced"
        );
    }
    harness.within(|| *MOOD.write() = Mood::Asleep);
    settle_until(&mut harness, |h| face(h).as_deref() == Some("sleeping"));
    assert_eq!(frame(&harness).as_deref(), Some("0"));
}

#[allow(non_snake_case)]
fn StillStage() -> Element {
    rsx! {
        Ds { appearance: Appearance { theme: Theme::Light, ..Appearance::default() }, material: Material::Window,
            AnimatedEmoji { emoji: EmojiId::Wink, size: PersonaSize::Medium, playback: EmojiPlayback::Still }
        }
    }
}

#[test]
fn a_still_picture_never_leaves_its_rest_frame() {
    let mut harness = Harness::new(StillStage, VIEW);
    for _ in 0..10 {
        harness.advance(Duration::from_millis(100));
        assert_eq!(
            frame(&harness).as_deref(),
            Some("0"),
            "a still picture moved"
        );
    }
    assert!(!harness.is_animating());
}
