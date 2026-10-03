//! Scrubber: a recording's progress bar at rest, with loaded stretches, in each pose, and
//! unavailable. The first one is live: press and drag it, hover for the time, use the arrows.

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::controls::scrubber::Scrubber;
use ds::components::controls::scrubber_face::ScrubberFace;
use ds::components::controls::scrubber_model::{BufferedRange, ScrubPose};
use ds::motion::spring::Millis;
use ds::prelude::*;

const LENGTH: Millis = Millis(3_723_000);

/// The Scrubber section.
#[component]
pub fn ScrubberSection() -> Element {
    let mut position = use_signal(|| Fraction(350));
    let loaded = vec![
        BufferedRange {
            from: Fraction(0),
            to: Fraction(520),
        },
        BufferedRange {
            from: Fraction(700),
            to: Fraction(800),
        },
    ];
    rsx! {
        Section { title: "Scrubber", note: "A Slider for a recording: loaded stretches behind the played fill, a tooltip reading the time under the pointer, and a drag that keeps following the pointer outside the bar. Escape abandons a drag; the arrows step, Shift steps far.",
            div { class: "g-grid2",
                Specimen { name: "live".to_owned(),
                    div { style: "width:260px; padding-top:28px",
                        Scrubber {
                            label: "Position",
                            position: position(),
                            length: LENGTH,
                            buffered: loaded.clone(),
                            onscrubstart: move |at| position.set(at),
                            onscrub: move |at| position.set(at),
                            onseek: move |at| position.set(at),
                        }
                    }
                }
                Specimen { name: "at rest".to_owned(),
                    div { style: "width:260px; padding-top:28px",
                        ScrubberFace { label: "Position", position: Fraction(350), length: LENGTH }
                    }
                }
                Specimen { name: "hover".to_owned(),
                    div { style: "width:260px; padding-top:28px",
                        ScrubberFace { label: "Position", position: Fraction(350), length: LENGTH, pose: ScrubPose::Hover, pointer: Fraction(500) }
                    }
                }
                Specimen { name: "dragging".to_owned(),
                    div { style: "width:260px; padding-top:28px",
                        ScrubberFace { label: "Position", position: Fraction(350), length: LENGTH, pose: ScrubPose::Dragging, pointer: Fraction(750) }
                    }
                }
                Specimen { name: "disabled".to_owned(),
                    div { style: "width:260px; padding-top:28px",
                        ScrubberFace { label: "Position", position: Fraction(350), length: LENGTH, availability: Availability::Disabled }
                    }
                }
                Specimen { name: "busy".to_owned(),
                    div { style: "width:260px; padding-top:28px",
                        ScrubberFace { label: "Position", position: Fraction(350), length: LENGTH, availability: Availability::Busy }
                    }
                }
            }
        }
    }
}
