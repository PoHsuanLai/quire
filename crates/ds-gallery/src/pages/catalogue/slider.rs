//! Slider: the linear look at each size and with ticks, the two capsule looks, disabled and busy.

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::Word;
use ds::{Availability, ControlSize, Fraction, LevelGlyph, Muting, Slider, SliderLook, Ticks};

/// The Slider section.
#[component]
pub fn SliderSection() -> Element {
    let mut linear = use_signal(|| Fraction(400));
    let mut ticked = use_signal(|| Fraction(500));
    let mut volume = use_signal(|| Fraction(600));
    let mut brightness = use_signal(|| Fraction(300));
    rsx! {
        Section { title: "Slider", note: "NSSlider: drag, click or use the arrows; with ticks the value snaps to a mark and the knob springs to it on release; the capsule looks carry a glyph that follows the level.",
            div { class: "g-grid2",
                for size in ControlSize::ALL.iter().copied() {
                    Specimen { name: format!("linear {}", size.slug()),
                        div { style: "width:220px",
                            Slider { label: "Level", value: linear(), size, onchange: move |next| linear.set(next) }
                        }
                    }
                }
                Specimen { name: "ticks every quarter".to_owned(), code: "Ticks::Every(250)".to_owned(),
                    div { style: "width:220px",
                        Slider { label: "Level", value: ticked(), ticks: Ticks::Every(Fraction(250)), onchange: move |next| ticked.set(next) }
                    }
                }
                Specimen { name: "disabled".to_owned(),
                    div { style: "width:220px",
                        Slider { label: "Level", value: Fraction(600), availability: Availability::Disabled }
                    }
                }
                Specimen { name: "busy".to_owned(),
                    div { style: "width:220px",
                        Slider { label: "Level", value: Fraction(600), availability: Availability::Busy }
                    }
                }
            }
            div { class: "g-grid2",
                for look in [SliderLook::Capsule, SliderLook::CapsuleKnob] {
                    Specimen { name: look.slug().to_owned(),
                        div { style: "width:260px; display:flex; flex-direction:column; gap:12px",
                            Slider { label: "Volume", value: volume(), look, glyph: LevelGlyph::Volume(Muting::Audible), onchange: move |next| volume.set(next) }
                            Slider { label: "Brightness", value: brightness(), look, glyph: LevelGlyph::Brightness, onchange: move |next| brightness.set(next) }
                            Slider { label: "Muted", value: Fraction(400), look, glyph: LevelGlyph::Volume(Muting::Muted), onchange: |_| {} }
                            Slider { label: "Disabled", value: Fraction(0), look, glyph: LevelGlyph::Brightness, availability: Availability::Disabled }
                        }
                    }
                }
            }
        }
    }
}
