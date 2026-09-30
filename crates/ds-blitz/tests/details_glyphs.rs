//! design/26 D0b's glyph morphs on a real Blitz document: MorphGlyph stacks the outgoing and
//! incoming layers only while the cross-fade plays, draws its slash on, and snaps under Reduced;
//! each ends at 0 frames (R2, R3, R7).

use dioxus::prelude::*;
use ds::detail::{MorphGlyph, MorphStyle, Slashed};
use ds::{Appearance, Ds, Icon, IconSize, Material, Motion};
use ds_blitz::harness::{assert_settles_to_zero_frames, settle_until};
use ds_blitz::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 200,
    height: 120,
    scale_percent: 100,
};

static ICON: GlobalSignal<Icon> = Signal::global(|| Icon::Volume);
static SLASHED: GlobalSignal<Slashed> = Signal::global(|| Slashed::Off);
static MOTION: GlobalSignal<Motion> = Signal::global(|| Motion::Standard);

#[allow(non_snake_case)]
fn Glyphs() -> Element {
    rsx! {
        Ds { appearance: Appearance { motion: MOTION(), ..Appearance::default() }, material: Material::Window,
            div { id: "down", MorphGlyph { icon: ICON(), size: IconSize::Base, style: MorphStyle::CrossFade } }
            div { id: "slash", MorphGlyph { icon: Icon::Volume, size: IconSize::Base, style: MorphStyle::Slash, slashed: SLASHED() } }
        }
    }
}

#[test]
fn a_cross_fade_stacks_two_layers_only_while_it_plays() {
    let mut harness = Harness::new(Glyphs, VIEW);
    assert_eq!(harness.count("#down .ds-morph-layer"), 1);
    // The same icon again is no morph (R2).
    harness.within(|| *ICON.write() = Icon::Volume);
    harness.advance(Duration::from_millis(30));
    assert_eq!(harness.count("#down .ds-morph-layer"), 1);
    harness.within(|| *ICON.write() = Icon::VolumeX);
    settle_until(&mut harness, |h| h.count("#down [*|data-morph=out]") == 1);
    assert!(harness.has_class("#down [*|data-morph=in]", "a-morph-fade-in"));
    assert!(harness.has_class("#down [*|data-morph=out]", "a-morph-fade-out"));
    settle_until(&mut harness, |h| h.count("#down .ds-morph-layer") == 1);
    assert_eq!(
        harness
            .attr("#down .ds-morph-layer", "data-morph")
            .as_deref(),
        Some("still")
    );
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn a_slash_draws_on_and_off() {
    // On `Clock::Virtual`: a known flaker under load, since the CSS stroke-dashoffset
    // resolved at the harness's frame clock could drift from the settle timer's real wall clock,
    // so `settle_until`'s poll for a mid-draw offset (`> 0.0`) could land after the draw had
    // already settled to 0.
    let mut harness =
        Harness::with_config(Glyphs, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    assert_eq!(harness.count("#slash .ds-morph-slash"), 0);
    harness.within(|| *SLASHED.write() = Slashed::On);
    let offset = |h: &Harness| {
        h.attr("#slash .ds-morph-slash path", "stroke-dashoffset")
            .and_then(|value| value.parse::<f32>().ok())
    };
    settle_until(&mut harness, |h| offset(h).is_some_and(|o| o > 0.0));
    settle_until(&mut harness, |h| offset(h) == Some(0.0));
    assert_settles_to_zero_frames(&mut harness);
    harness.within(|| *SLASHED.write() = Slashed::Off);
    settle_until(&mut harness, |h| h.count("#slash .ds-morph-slash") == 0);
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn reduced_snaps_a_morph() {
    let mut harness = Harness::new(Glyphs, VIEW);
    harness.within(|| *MOTION.write() = Motion::Reduced);
    harness.advance(Duration::from_millis(20));
    harness.within(|| *ICON.write() = Icon::VolumeX);
    harness.advance(Duration::from_millis(40));
    assert_eq!(harness.count("#down .ds-morph-layer"), 1);
    assert_eq!(harness.count("#down [*|data-morph=out]"), 0);
    assert_settles_to_zero_frames(&mut harness);
}
