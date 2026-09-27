//! Details, the control center's modules (design/26-DETAILS.md 5.2, wave D2): the tile disc's
//! fill and morph, the settings rows' pending, success and failure, Now Playing's play/pause,
//! track cross-fade and position, the Battery module's rings, and the keyboard-brightness level.
//! Each cell opens at rest (a snapshot runs no Rust timer); its buttons play the moments.

use super::Section;
use super::details::{Cell, mini};
use super::details_center_rows::{DeviceRows, NetworkRows, OutputRows};
use dioxus::prelude::*;
use ds::detail::{EventStamp, FirstShow};
use ds::{
    DeviceBattery, DiscMotion, Fraction, Glyph, Icon, IconButton, IconButtonVariant, IconSize,
    LevelControl, LevelGlyph, LevelLook, LevelMode, ModuleGrid, ModulePanel, ModuleState,
    ModuleTile, NowPlayingTrack, PlayPauseButton, Playback, Px, RingMark, Text, Tick,
    TrackPosition,
};
use std::time::Duration;

/// The section.
#[component]
pub fn CenterSection() -> Element {
    rsx! {
        Section { title: "Control center modules", note: "The control center's details (D2). A tile's disc fills its glyph once as the module comes on (Wi-Fi) or morphs into its on glyph (Focus, springing only under a press). A row joining shows a spinner where its lock was, a device connecting or an output switching breathes its glyph; success seals the disc or draws the check once; failure shakes the row once. Play/pause offers the next action off-up; a new track cross-fades; the position steps once a second while playing. The Battery module's rings sweep in with their percentages counting in step. Keyboard brightness is a level whose rays follow it. Nothing loops; each settles to 0 frames.",
            div { class: "g-detail-grid g-center-grid",
                TilesCell {}
                NetworkRows {}
                DeviceRows {}
                OutputRows {}
                PlayerCell {}
                BatteryCell {}
                KeyboardCell {}
            }
        }
    }
}

/// On and off, as a tile's state.
fn flip(state: ModuleState) -> ModuleState {
    match state {
        ModuleState::Off => ModuleState::On,
        ModuleState::On | ModuleState::Busy => ModuleState::Off,
    }
}

#[component]
fn TilesCell() -> Element {
    let mut wifi = use_signal(|| ModuleState::On);
    let mut focus = use_signal(|| ModuleState::Off);
    rsx! {
        Cell { name: "Tile disc", code: "ModuleTile {{ disc: DiscMotion::Fill | Morph(icon) }}",
            controls: rsx! {
                {mini("Wi-Fi busy", move |_| wifi.set(ModuleState::Busy))}
                {mini("Wi-Fi on", move |_| wifi.set(ModuleState::On))}
                {mini("Focus from elsewhere", move |_| focus.set(flip(focus())))}
            },
            div { class: "g-detail",
                div { class: "g-center-tiles",
                ModuleGrid { padding: Px(0.0),
                    ModuleTile {
                        glyph: Icon::Wifi,
                        title: "Wi-Fi",
                        status: Some(Text::from(words(wifi()))),
                        state: wifi(),
                        disc: DiscMotion::Fill,
                        onclick: move |_| wifi.set(flip(wifi())),
                    }
                    ModuleTile {
                        glyph: Icon::Moon,
                        title: "Focus",
                        status: Some(Text::from(words(focus()))),
                        state: focus(),
                        disc: DiscMotion::Morph(Icon::MoonFilled),
                        onclick: move |_| focus.set(flip(focus())),
                    }
                }
                }
            }
        }
    }
}

/// A tile's status words (R8).
fn words(state: ModuleState) -> &'static str {
    match state {
        ModuleState::Off => "Off",
        ModuleState::On => "On",
        ModuleState::Busy => "Turning on…",
    }
}

/// The two tracks the player swaps between.
const TRACKS: [(&str, &str); 2] = [
    ("Clair de lune", "Claude Debussy"),
    ("Gymnopédie No. 1", "Erik Satie"),
];

#[component]
fn PlayerCell() -> Element {
    let mut playback = use_signal(|| Playback::Paused);
    let mut track = use_signal(|| 0usize);
    let mut waits = use_signal(|| 0u32);
    let (title, by) = TRACKS[track() % TRACKS.len()];
    rsx! {
        Cell { name: "Now Playing", code: "PlayPauseButton, NowPlayingTrack, TrackPosition",
            controls: rsx! {
                {mini("Next track", move |_| *track.write() += 1)}
                {mini("Buffer", move |_| { *waits.write() += 1; playback.set(Playback::Buffering(EventStamp(waits()))) })}
                {mini("Playing", move |_| playback.set(Playback::Playing))}
            },
            div { class: "g-detail g-detail-list",
                div { class: "g-center-player",
                    div { class: "g-center-track",
                        NowPlayingTrack { title: Text::from(title), by: Some(Text::from(by)), playback: playback() }
                    }
                    IconButton { variant: IconButtonVariant::Tool, icon: Icon::SkipBack, label: "Previous".to_owned(), onclick: move |_| *track.write() += 1 }
                    PlayPauseButton {
                        playback: playback(),
                        onclick: move |_| playback.set(match playback() {
                            Playback::Paused => Playback::Playing,
                            Playback::Playing | Playback::Buffering(_) => Playback::Paused,
                        }),
                    }
                    IconButton { variant: IconButtonVariant::Tool, icon: Icon::SkipForward, label: "Next".to_owned(), onclick: move |_| *track.write() += 1 }
                }
                TrackPosition { at: Duration::from_secs(83), length: Duration::from_secs(301), playback: playback() }
            }
        }
    }
}

#[component]
fn BatteryCell() -> Element {
    let mut appear = use_signal(|| 0u32);
    let mut mouse = use_signal(|| Fraction(640));
    let first = match appear() {
        0 => FirstShow::Still,
        _ => FirstShow::Animate,
    };
    rsx! {
        Cell { name: "Battery module", code: "DeviceBattery {{ level, mark, first }}",
            controls: rsx! {
                {mini("Open the center", move |_| *appear.write() += 1)}
                {mini("Mouse 64 %", move |_| mouse.set(Fraction(640)))}
                {mini("Mouse 31 %", move |_| mouse.set(Fraction(310)))}
            },
            for round in [appear()] {
                div { key: "{round}", class: "g-detail",
                    DeviceBattery { level: Fraction(930), mark: RingMark::Charging, label: "This computer", first,
                        Glyph { icon: Icon::Monitor, size: IconSize::Base }
                    }
                    DeviceBattery { level: mouse(), label: "Mouse", first,
                        Glyph { icon: Icon::Mouse, size: IconSize::Base }
                    }
                    DeviceBattery { level: Fraction(150), label: "Headphones", first,
                        Glyph { icon: Icon::Headphones, size: IconSize::Base }
                    }
                }
            }
        }
    }
}

#[component]
fn KeyboardCell() -> Element {
    let mut level = use_signal(|| Fraction(450));
    rsx! {
        Cell { name: "Keyboard brightness", code: "LevelControl {{ glyph: LevelGlyph::KeyboardBrightness }}",
            controls: rsx! {
                {mini("Dim", move |_| level.set(Fraction(100)))}
                {mini("Bright", move |_| level.set(Fraction(1000)))}
            },
            div { class: "g-detail g-detail-list",
                ModulePanel { glyph: Some(Icon::Keyboard), title: Some(Text::from("Keyboard Brightness")), trailing: rsx! { "{(level().0 + 5) / 10}%" },
                    LevelControl {
                        label: "Keyboard Brightness".to_owned(),
                        value: level(),
                        glyph: LevelGlyph::KeyboardBrightness,
                        mode: LevelMode::Interactive,
                        look: LevelLook::CapsuleKnob,
                        tick: Tick::Quiet,
                        onchange: move |next| level.set(next),
                    }
                }
            }
        }
    }
}
