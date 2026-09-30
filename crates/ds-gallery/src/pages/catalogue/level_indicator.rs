//! LevelIndicator: the continuous capsule and the sixteen steps, with a glyph and the warning and
//! critical bands, at each size.

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::content::level_glyph::vocab::LevelGlyph;
use ds::components::controls::level_indicator::{Bands, LevelIndicator, LevelStyle};
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;
use ds_core::vocab::Muting;

/// The LevelIndicator section.
#[component]
pub fn LevelIndicatorSection() -> Element {
    let bands = Bands {
        warning: Fraction(300),
        critical: Fraction(150),
    };
    rsx! {
        Section { title: "LevelIndicator", note: "NSLevelIndicator: read-only; the fill slides linearly over --t-move; at or under the warning level it draws in the warning ink and at or under the critical level in the critical one.",
            for style in LevelStyle::ALL.iter().copied() {
                div { class: "g-row g-row-top",
                    span { class: "g-name g-type-name", "{style.slug()}" }
                    for size in [ControlSize::Small, ControlSize::Regular] {
                        Specimen { name: format!("volume 60, {}", size.slug()),
                            div { style: "width:220px",
                                LevelIndicator { label: "Volume", value: Fraction(600), style, size, glyph: LevelGlyph::Volume(Muting::Audible) }
                            }
                        }
                    }
                    Specimen { name: "muted".to_owned(),
                        div { style: "width:220px",
                            LevelIndicator { label: "Volume", value: Fraction(600), style, glyph: LevelGlyph::Volume(Muting::Muted) }
                        }
                    }
                    Specimen { name: "brightness 30".to_owned(),
                        div { style: "width:220px",
                            LevelIndicator { label: "Brightness", value: Fraction(300), style, glyph: LevelGlyph::Brightness }
                        }
                    }
                }
                div { class: "g-row g-row-top",
                    span { class: "g-name g-type-name", "{style.slug()} bands" }
                    for (name , value) in [("normal 80", 800u16), ("warning 25", 250), ("critical 10", 100)] {
                        Specimen { name: name.to_owned(),
                            div { style: "width:220px",
                                LevelIndicator { label: "Battery", value: Fraction(value), style, bands }
                            }
                        }
                    }
                }
            }
        }
    }
}
