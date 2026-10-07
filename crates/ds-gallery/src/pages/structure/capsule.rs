//! Capsule: the zoom bar and the media player's controls, each over a stand-in for the content it
//! floats over. The media capsule is live: press play, drag the bar, move the level.

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::chrome::capsule::model::{CapsuleSlot, LevelSlot, ScrubEvent, ScrubSlot};
use ds::components::chrome::capsule::priority::essentials;
use ds::components::chrome::capsule::view::Capsule;
use ds::components::controls::scrubber_model::BufferedRange;
use ds::motion::spring::Millis;
use ds::prelude::*;

const LENGTH: Millis = Millis(180_000);

/// The Capsule section.
#[component]
pub fn CapsuleSection() -> Element {
    let mut zoom = use_signal(|| 100_u32);
    let mut position = use_signal(|| Fraction(350));
    let mut level = use_signal(|| Fraction(700));
    let mut playing = use_signal(|| Check::Off);
    let slots = vec![
        CapsuleSlot::Item(ds::components::chrome::toolbar::model::ToolbarItem::new(
            0,
            "Play",
            if playing() == Check::On {
                Icon::Pause
            } else {
                Icon::Play
            },
        ))
        .essential(),
        CapsuleSlot::Readout(clock(position())).droppable(1),
        CapsuleSlot::Scrub(ScrubSlot {
            label: "Position".to_owned(),
            position: position(),
            length: LENGTH,
            buffered: vec![BufferedRange {
                from: Fraction(0),
                to: Fraction(620),
            }],
            availability: Availability::Enabled,
        })
        .essential(),
        CapsuleSlot::Level(LevelSlot {
            label: "Volume".to_owned(),
            value: level(),
            availability: Availability::Enabled,
        })
        .droppable(2),
    ];
    rsx! {
        Section { title: "Capsule", note: "The pill of controls over content: toolbar buttons and readouts, and for a recording a progress bar that fills the free width and a fixed-width level.",
            div { class: "g-grid2",
                Specimen { name: "zoom".to_owned(),
                    div { style: "position:relative; width:360px; height:120px",
                        Capsule::<u8> {
                            label: "Zoom",
                            slots: essentials(vec![
                                CapsuleSlot::button(0, "Zoom out", Icon::Minus),
                                CapsuleSlot::Readout(format!("{}%", zoom())),
                                CapsuleSlot::button(1, "Zoom in", Icon::Plus),
                            ]),
                            shown: Shown::Visible,
                            onpick: move |value| zoom.set(if value == 0 { zoom().saturating_sub(25) } else { zoom() + 25 }),
                        }
                    }
                }
                Specimen { name: "media, wide".to_owned(),
                    div { style: "position:relative; width:480px; height:140px",
                        Capsule::<u8> {
                            label: "Playback",
                            slots: slots.clone(),
                            shown: Shown::Visible,
                            onpick: move |_| playing.set(if playing() == Check::On { Check::Off } else { Check::On }),
                            onscrub: move |event| match event {
                                ScrubEvent::Start(at) | ScrubEvent::Move(at) | ScrubEvent::End(at) | ScrubEvent::Seek(at) => position.set(at),
                                ScrubEvent::Cancel => {}
                            },
                            onlevel: move |next| level.set(next),
                        }
                    }
                }
                Specimen { name: "media, narrow (the clock and level drop)".to_owned(),
                    div { style: "position:relative; width:300px; height:140px",
                        Capsule::<u8> {
                            label: "Playback",
                            slots: slots.clone(),
                            shown: Shown::Visible,
                            onpick: move |_| playing.set(if playing() == Check::On { Check::Off } else { Check::On }),
                            onscrub: move |event| match event {
                                ScrubEvent::Start(at) | ScrubEvent::Move(at) | ScrubEvent::End(at) | ScrubEvent::Seek(at) => position.set(at),
                                ScrubEvent::Cancel => {}
                            },
                            onlevel: move |next| level.set(next),
                        }
                    }
                }
            }
        }
    }
}

/// `m:ss` of the way through the recording.
fn clock(at: Fraction) -> String {
    ds::components::controls::scrubber_model::time_text(LENGTH, at)
}
