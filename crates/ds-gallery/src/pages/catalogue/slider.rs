//! Slider: the linear look at each size and with ticks, the level look, disabled and busy.

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::content::level_glyph::vocab::LevelGlyph;
use ds::components::controls::slider_model::SliderLook;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;
use ds_core::vocab::Muting;

/// The Slider section.
#[component]
pub fn SliderSection() -> Element {
    let mut linear = use_signal(|| Fraction(400));
    let mut ticked = use_signal(|| Fraction(500));
    let mut volume = use_signal(|| Fraction(600));
    let mut brightness = use_signal(|| Fraction(300));
    rsx! {
        Section { title: "Slider", note: "NSSlider: drag, click or use the arrows; with ticks the value snaps to a mark and the knob springs to it on release; the level look carries a glyph that follows the level.",
            div { class: "g-grid2",
                for size in ControlSize::ALL.iter().copied() {
                    Specimen { name: format!("linear {}", size.slug()),
                        div { style: "width:220px",
                            Slider { label: "Level", value: linear(), size, onchange: move |next| linear.set(next) }
                        }
                    }
                }
                Specimen { name: "ticks every quarter".to_owned(), code: "ticks: Some(Fraction(250))".to_owned(),
                    div { style: "width:220px",
                        Slider { label: "Level", value: ticked(), ticks: Some(Fraction(250)), onchange: move |next| ticked.set(next) }
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
                Specimen { name: SliderLook::Level.slug().to_owned(),
                    div { style: "width:260px; display:flex; flex-direction:column; gap:12px",
                        Slider { label: "Volume", value: volume(), look: SliderLook::Level, glyph: LevelGlyph::Volume(Muting::Audible), onchange: move |next| volume.set(next) }
                        Slider { label: "Brightness", value: brightness(), look: SliderLook::Level, glyph: LevelGlyph::Brightness, onchange: move |next| brightness.set(next) }
                        Slider { label: "Muted", value: Fraction(400), look: SliderLook::Level, glyph: LevelGlyph::Volume(Muting::Muted), onchange: |_| {} }
                        Slider { label: "Disabled", value: Fraction(0), look: SliderLook::Level, glyph: LevelGlyph::Brightness, availability: Availability::Disabled }
                    }
                }
            }
        }
    }
}
