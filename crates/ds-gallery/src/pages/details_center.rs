//! Details, the control center's modules (design/26-DETAILS.md 5.2): the tile disc's busy ring,
//! the settings rows' pending, Now Playing's play/pause, track cross-fade and position, the
//! Battery module's rings, and the keyboard-brightness level. Each cell opens at rest (a
//! snapshot runs no Rust timer); its buttons play the moments.

use super::Section;
use super::details::{Cell, mini};
use super::details_center_rows::{DeviceRows, NetworkRows, OutputRows};
use dioxus::prelude::*;
use ds::detail::EventStamp;
use ds::{Bezel, Button, ControlSize, IconSwap, ImagePosition};
use ds::{Fraction, Glyph, Icon, IconSize, LevelGlyph, Px, TextLine};
use ds::{Slider, SliderLook};
use ds_shell::{
    DeviceBattery, ModuleGrid, ModulePanel, ModuleState, ModuleTile, NowPlayingTrack, Playback,
    RingMark, TrackPosition,
};
use std::time::Duration;

/// The section.
#[component]
pub fn CenterSection() -> Element {
    rsx! {
        Section { title: "Control center modules", note: "The control center's details (D2). A busy tile shows the spinner ring on its disc. A row joining shows a spinner where its lock was. Play/pause cross-fades to the next action; a new track cross-fades; the position steps once a second while playing. The Battery module's rings follow their levels. Keyboard brightness is a level whose rays follow it.",
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
        Cell { name: "Tile disc", code: "ModuleTile {{ state }}",
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
                        status: Some(TextLine::from(words(wifi()))),
                        state: wifi(),
                        onclick: move |_| wifi.set(flip(wifi())),
                    }
                    ModuleTile {
                        glyph: Icon::Moon,
                        title: "Focus",
                        status: Some(TextLine::from(words(focus()))),
                        state: focus(),
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
        Cell { name: "Now Playing", code: "NowPlayingTrack, TrackPosition",
            controls: rsx! {
                {mini("Next track", move |_| *track.write() += 1)}
                {mini("Buffer", move |_| { *waits.write() += 1; playback.set(Playback::Buffering(EventStamp(waits()))) })}
                {mini("Playing", move |_| playback.set(Playback::Playing))}
            },
            div { class: "g-detail g-detail-list",
                div { class: "g-center-player",
                    div { class: "g-center-track",
                        NowPlayingTrack { title: TextLine::from(title), by: Some(TextLine::from(by)) }
                    }
                    Button { bezel: Bezel::Toolbar, size: ControlSize::Large, image: ImagePosition::Only, icon: Icon::SkipBack, label: "Previous".to_owned(), onclick: move |_| *track.write() += 1 }
                    Button {
                        bezel: Bezel::Toolbar,
                        size: ControlSize::Large,
                        image: ImagePosition::Only,
                        swap: IconSwap::CrossFade,
                        icon: playback().next_action(),
                        label: playback().label(),
                        onclick: move |_| playback.set(match playback() {
                            Playback::Paused => Playback::Playing,
                            Playback::Playing | Playback::Buffering(_) => Playback::Paused,
                        }),
                    }
                    Button { bezel: Bezel::Toolbar, size: ControlSize::Large, image: ImagePosition::Only, icon: Icon::SkipForward, label: "Next".to_owned(), onclick: move |_| *track.write() += 1 }
                }
                TrackPosition { at: Duration::from_secs(83), length: Duration::from_secs(301), playback: playback() }
            }
        }
    }
}

#[component]
fn BatteryCell() -> Element {
    let mut mouse = use_signal(|| Fraction(640));
    rsx! {
        Cell { name: "Battery module", code: "DeviceBattery {{ level, mark }}",
            controls: rsx! {
                {mini("Mouse 64 %", move |_| mouse.set(Fraction(640)))}
                {mini("Mouse 31 %", move |_| mouse.set(Fraction(310)))}
            },
            div { class: "g-detail",
                DeviceBattery { level: Fraction(930), mark: RingMark::Charging, label: "This computer",
                    Glyph { icon: Icon::Monitor, size: IconSize::Base }
                }
                DeviceBattery { level: mouse(), label: "Mouse",
                    Glyph { icon: Icon::Mouse, size: IconSize::Base }
                }
                DeviceBattery { level: Fraction(150), label: "Headphones",
                    Glyph { icon: Icon::Headphones, size: IconSize::Base }
                }
            }
        }
    }
}

#[component]
fn KeyboardCell() -> Element {
    let mut level = use_signal(|| Fraction(450));
    rsx! {
        Cell { name: "Keyboard brightness", code: "Slider {{ glyph: LevelGlyph::KeyboardBrightness, look: CapsuleKnob }}",
            controls: rsx! {
                {mini("Dim", move |_| level.set(Fraction(100)))}
                {mini("Bright", move |_| level.set(Fraction(1000)))}
            },
            div { class: "g-detail g-detail-list",
                ModulePanel { glyph: Some(Icon::Keyboard), title: Some(TextLine::from("Keyboard Brightness")), trailing: rsx! { "{(level().0 + 5) / 10}%" },
                    Slider {
                        label: "Keyboard Brightness".to_owned(),
                        value: level(),
                        glyph: LevelGlyph::KeyboardBrightness,
                        look: SliderLook::CapsuleKnob,
                        onchange: move |next| level.set(next),
                    }
                }
            }
        }
    }
}
