//! design/26 D1 on a real Blitz document: the Bluetooth status glyph (G6, G7: the slash, the
//! bounded breath while connecting, the seal and dots on connecting, the shake) and the volume
//! glyph on `LevelGlyph` (G12); each ends at 0 frames (R3).

use dioxus::prelude::*;
use ds::detail::EventStamp;
use ds::{
    Appearance, BluetoothGlyph, BluetoothState, Ds, Material, Motion, VolumeGlyph, VolumeState,
    VolumeWaves,
};
use ds_native::harness::{assert_settles_to_zero_frames, settle_until};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 160,
    height: 80,
    scale_percent: 100,
};

static BLUETOOTH: GlobalSignal<BluetoothState> = Signal::global(|| BluetoothState::On);
static VOLUME: GlobalSignal<VolumeState> =
    Signal::global(|| VolumeState::Heard(VolumeWaves::Three));
static MOTION: GlobalSignal<Motion> = Signal::global(|| Motion::Standard);

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance { motion: MOTION(), ..Appearance::default() }, material: Material::Window,
            div { id: "bt", BluetoothGlyph { state: BLUETOOTH() } }
            div { id: "vol", VolumeGlyph { state: VOLUME() } }
        }
    }
}

fn set(harness: &mut Harness, state: BluetoothState) {
    harness.within(|| *BLUETOOTH.write() = state);
}

fn pending(harness: &Harness) -> Option<String> {
    harness.attr("#bt .ds-status-glyph", "data-pending")
}

fn dots(harness: &Harness) -> Option<String> {
    harness.attr("#bt [*|data-part=dots]", "data-show")
}

#[test]
fn connecting_breathes_after_its_grace_and_a_connection_seals_once_with_its_dots() {
    let mut harness = Harness::new(Page, VIEW);
    assert_eq!(dots(&harness).as_deref(), Some("hidden"));
    set(&mut harness, BluetoothState::Connecting(EventStamp(1)));
    harness.advance(Duration::from_millis(20));
    assert_eq!(pending(&harness).as_deref(), Some("idle"));
    settle_until(&mut harness, |h| pending(h).as_deref() == Some("high"));
    settle_until(&mut harness, |h| pending(h).as_deref() == Some("low"));
    set(&mut harness, BluetoothState::Connected);
    settle_until(&mut harness, |h| {
        h.has_class("#bt .ds-status-glyph", "a-seal-out")
    });
    assert!(
        !harness.has_class("#bt .ds-status-glyph", "a-gulp"),
        "a connection the service made sprang (R5)"
    );
    assert_eq!(pending(&harness).as_deref(), Some("idle"));
    settle_until(&mut harness, |h| dots(h).as_deref() == Some("lit"));
    settle_until(&mut harness, |h| {
        !h.has_class("#bt .ds-status-glyph", "a-seal-out")
    });
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn bluetooth_off_draws_the_slash_and_a_failure_shakes_once() {
    let mut harness = Harness::new(Page, VIEW);
    set(&mut harness, BluetoothState::Off);
    settle_until(&mut harness, |h| h.count("#bt [*|data-part=slash]") == 1);
    assert_eq!(
        harness
            .attr("#bt [*|data-part=rune]", "data-show")
            .as_deref(),
        Some("faint")
    );
    assert_settles_to_zero_frames(&mut harness);
    set(&mut harness, BluetoothState::Failed(EventStamp(1)));
    settle_until(&mut harness, |h| {
        h.has_class("#bt .ds-status-glyph", "a-shake-x")
    });
    settle_until(&mut harness, |h| h.count("#bt [*|data-part=slash]") == 0);
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn reduced_holds_the_breath_still() {
    let mut harness = Harness::new(Page, VIEW);
    harness.within(|| *MOTION.write() = Motion::Reduced);
    harness.advance(Duration::from_millis(20));
    set(&mut harness, BluetoothState::Connecting(EventStamp(2)));
    settle_until(&mut harness, |h| pending(h).as_deref() == Some("still"));
    assert_settles_to_zero_frames(&mut harness);
}

fn part_on(harness: &Harness, part: &str) -> Option<String> {
    harness.attr(&format!("#vol [*|data-part={part}]"), "data-on")
}

#[test]
fn the_volume_waves_cross_fade_and_mute_brings_the_slash() {
    let mut harness = Harness::new(Page, VIEW);
    assert_eq!(part_on(&harness, "wave-3").as_deref(), Some("on"));
    assert_eq!(part_on(&harness, "slash").as_deref(), Some("off"));
    harness.within(|| *VOLUME.write() = VolumeState::Heard(VolumeWaves::One));
    settle_until(&mut harness, |h| {
        part_on(h, "wave-2").as_deref() == Some("off")
    });
    assert_eq!(part_on(&harness, "wave-1").as_deref(), Some("on"));
    assert_settles_to_zero_frames(&mut harness);
    harness.within(|| *VOLUME.write() = VolumeState::Muted);
    settle_until(&mut harness, |h| {
        part_on(h, "slash").as_deref() == Some("on")
    });
    assert_eq!(part_on(&harness, "wave-1").as_deref(), Some("off"));
    assert_settles_to_zero_frames(&mut harness);
    harness.within(|| *VOLUME.write() = VolumeState::NoDevice);
    harness.advance(Duration::from_millis(30));
    assert_eq!(part_on(&harness, "slash").as_deref(), Some("on"));
    assert_settles_to_zero_frames(&mut harness);
}
