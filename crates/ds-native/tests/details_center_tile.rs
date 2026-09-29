//! design/26 on a real Blitz document, on the virtual clock: a module tile's disc answering the
//! module coming on (the glyph's layers fill once; the Focus moon morphs into the filled
//! moon, springing only under the tile's own press), every moment ending at 0 frames (R3), and
//! under Reduced motion (R7).

use dioxus::prelude::*;
use ds::{Appearance, DiscMotion, Ds, Icon, Material, ModuleState, ModuleTile, Motion, Text};
use ds_native::harness::assert_settles_to_zero_frames;
use ds_native::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 360,
    height: 120,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn virtual_harness(app: fn() -> Element) -> Harness {
    Harness::with_config(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

static WIFI: GlobalSignal<ModuleState> = Signal::global(|| ModuleState::Off);
static FOCUS: GlobalSignal<ModuleState> = Signal::global(|| ModuleState::Off);
static MOTION: GlobalSignal<Motion> = Signal::global(|| Motion::Standard);

fn flip(state: ModuleState) -> ModuleState {
    match state {
        ModuleState::Off => ModuleState::On,
        ModuleState::On | ModuleState::Busy => ModuleState::Off,
    }
}

#[allow(non_snake_case)]
fn Tiles() -> Element {
    rsx! {
        Ds { appearance: Appearance { motion: MOTION(), ..Appearance::default() }, material: Material::Window,
            div { style: "display:flex; gap:8px; width:340px",
                div { id: "wifi", style: "flex:1",
                    ModuleTile { glyph: Icon::Wifi, title: "Wi-Fi", status: Some(Text::from("On")), state: WIFI(), disc: DiscMotion::Fill, onclick: move |_| *WIFI.write() = flip(WIFI()) }
                }
                div { id: "focus", style: "flex:1",
                    ModuleTile { glyph: Icon::Moon, title: "Focus", status: None, state: FOCUS(), disc: DiscMotion::Morph(Icon::MoonFilled), onclick: move |_| *FOCUS.write() = flip(FOCUS()) }
                }
            }
        }
    }
}

/// The Wi-Fi disc's lit layers.
fn lit(harness: &Harness) -> usize {
    harness.count("#wifi .ds-module-disc [*|data-lit=on]")
}

fn set(harness: &mut Harness, signal: &'static GlobalSignal<ModuleState>, state: ModuleState) {
    harness.within(|| *signal.write() = state);
}

#[test]
fn turning_on_fills_the_disc_glyph_once_from_the_dot_out() {
    let mut harness = virtual_harness(Tiles);
    assert_eq!(lit(&harness), 4, "{}", harness.html());
    assert_settles_to_zero_frames(&mut harness);
    set(&mut harness, &WIFI, ModuleState::On);
    // The first step lights only the dot (layer 0), then one more layer each --t-pending-step.
    harness.advance(ms(0));
    assert_eq!(lit(&harness), 1, "the fill starts from the dot");
    assert_eq!(
        harness.attr("#wifi .ds-module-disc [*|data-lit=on]", "data-layer"),
        Some("0".to_owned())
    );
    harness.advance(ms(300));
    assert_eq!(lit(&harness), 2);
    harness.advance(ms(600));
    assert_eq!(lit(&harness), 4);
    assert_settles_to_zero_frames(&mut harness);
    assert_eq!(lit(&harness), 4, "at rest the glyph is whole");
    // Off is a Change: no fill.
    set(&mut harness, &WIFI, ModuleState::Off);
    harness.advance(ms(0));
    assert_eq!(lit(&harness), 4);
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn a_busy_module_landing_on_fills_too_and_its_ring_is_bounded() {
    let mut harness = virtual_harness(Tiles);
    set(&mut harness, &WIFI, ModuleState::Busy);
    harness.advance(ms(500));
    assert_eq!(
        harness.attr("#wifi .ds-spinner", "data-pending"),
        Some("step".to_owned())
    );
    set(&mut harness, &WIFI, ModuleState::On);
    harness.advance(ms(0));
    assert_eq!(harness.count("#wifi .ds-spinner"), 0);
    assert_eq!(lit(&harness), 1);
    assert_settles_to_zero_frames(&mut harness);
    assert_eq!(lit(&harness), 4);
}

/// The Focus disc's incoming glyph's class, while a morph plays.
fn incoming(harness: &Harness) -> Option<String> {
    harness.attr("#focus [*|data-morph=in]", "class")
}

#[test]
fn the_focus_moon_morphs_to_the_filled_moon_springing_only_under_a_press() {
    let mut harness = virtual_harness(Tiles);
    assert_eq!(
        harness.count("#focus .ds-morph-glyph path[*|fill=currentColor]"),
        0
    );
    // From elsewhere (a schedule): the filled moon grows in at --e-out.
    set(&mut harness, &FOCUS, ModuleState::On);
    harness.advance(ms(0));
    let class = incoming(&harness).unwrap_or_default();
    assert!(
        class.contains("a-morph-in") && !class.contains("spring"),
        "{class}"
    );
    assert_eq!(
        harness.count("#focus .ds-morph-glyph [*|data-morph=in] path[*|fill=currentColor]"),
        1,
        "the incoming glyph is the filled moon"
    );
    assert_settles_to_zero_frames(&mut harness);
    // Off again from elsewhere: DownUp back to the outline, still no spring.
    set(&mut harness, &FOCUS, ModuleState::Off);
    harness.advance(ms(0));
    assert!(!incoming(&harness).unwrap_or_default().contains("spring"));
    assert_settles_to_zero_frames(&mut harness);
    // A press on the tile: the change it causes springs.
    let at = harness.centre("#focus .ds-module-title").expect("the tile");
    harness.click(at);
    harness.advance(ms(0));
    assert_eq!(harness.within(|| *FOCUS.read()), ModuleState::On);
    let class = incoming(&harness).unwrap_or_default();
    assert!(class.contains("a-morph-in-spring"), "{class}");
    assert_settles_to_zero_frames(&mut harness);
    // The press is spent: the next change from elsewhere does not spring.
    set(&mut harness, &FOCUS, ModuleState::Off);
    harness.advance(ms(0));
    assert!(!incoming(&harness).unwrap_or_default().contains("spring"));
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn reduced_shows_the_whole_glyph_and_snaps_the_morph() {
    let mut harness = virtual_harness(Tiles);
    harness.within(|| *MOTION.write() = Motion::Reduced);
    harness.advance(ms(20));
    set(&mut harness, &WIFI, ModuleState::On);
    set(&mut harness, &FOCUS, ModuleState::On);
    harness.advance(ms(0));
    assert_eq!(lit(&harness), 4, "no fill under Reduced");
    assert_eq!(
        harness.count("#focus [*|data-morph=in]"),
        0,
        "no morph under Reduced"
    );
    assert_eq!(
        harness.count("#focus .ds-morph-glyph path[*|fill=currentColor]"),
        1,
        "the filled moon at once"
    );
    assert_settles_to_zero_frames(&mut harness);
}
