//! The Voice orb page: the three demo variants (default 192 px, small 96 px, custom colours at
//! 128 px with a 15 s period), a size ladder that crosses every metric threshold, and a switch
//! that puts them all at rest or sets them turning.

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::content::voice_orb::model::{OrbColour, OrbColours};
use ds::components::content::voice_orb::view::{ORB_PERIOD, VoiceOrb};
use ds::prelude::*;
use ds_core::vocab::Activity;
use ds_style::tokens::control_size::ControlSize;
use ds_style::tokens::hex::Hex;
use std::time::Duration;

/// The sizes of the ladder: under 30 (no mask), 30 to 50, 50 to 100 and 100 and over.
const LADDER: [f32; 6] = [24.0, 40.0, 64.0, 96.0, 128.0, 192.0];

/// The caller's own colours for the custom variant: a warm ground and three glows.
const WARM: OrbColours = OrbColours {
    bg: OrbColour::Custom(Hex([0xF6, 0xF1, 0xE7])),
    c1: OrbColour::Custom(Hex([0xE8, 0x8B, 0x5A])),
    c2: OrbColour::Custom(Hex([0x6B, 0xB3, 0x9C])),
    c3: OrbColour::Custom(Hex([0xE3, 0xC0, 0x5B])),
};

/// The page.
#[component]
pub fn VoiceOrbPage() -> Element {
    let mut activity = use_signal(|| Activity::Active);
    let next = match activity() {
        Activity::Active => Activity::Inactive,
        Activity::Inactive => Activity::Active,
    };
    rsx! {
        Section { title: "Variants", note: "The three demo orbs. While active the glows turn once per period (20 s, 15 s for the custom one); at rest they hold where they stand and nothing runs. Reduced motion holds them still. Blitz paints the blur and the contrast on both renderers; the dot grid is a plain layer at reduced opacity, where a browser overlay-blends it.",
            div { class: "g-row",
                Button {
                    size: ControlSize::Mini,
                    label: format!("Set {}", next.slug()),
                    onclick: move |_| activity.set(next),
                }
                span { class: "g-code", "activity: {activity().slug()}" }
            }
            div { class: "g-row g-row-top",
                Specimen { name: "Default", code: "192 px, 20 s".to_owned(),
                    VoiceOrb { activity: activity() }
                }
                Specimen { name: "Small", code: "96 px".to_owned(),
                    VoiceOrb { size: Px(96.0), activity: activity() }
                }
                Specimen { name: "Custom colours", code: "128 px, 15 s".to_owned(),
                    VoiceOrb { size: Px(128.0), colours: WARM, period: Duration::from_secs(15), activity: activity() }
                }
            }
        }
        Section { title: "Sizes", note: "Every size threshold: under 30 px the dot mask is off and contrast is lowest; 30 to 50 a 5 % mask; 50 to 100 a 15 % mask; 100 and up 25 %.",
            div { class: "g-row g-row-top",
                for size in LADDER {
                    Specimen { name: format!("{size} px"), code: None,
                        VoiceOrb { size: Px(size), period: ORB_PERIOD, activity: activity() }
                    }
                }
            }
        }
    }
}
