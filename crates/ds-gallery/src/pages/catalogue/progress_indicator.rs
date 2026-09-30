//! ProgressIndicator: the bar, the spoke spinner and the ring, determinate and running, at each
//! size.

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::controls::progress::model::{Progress, ProgressStyle};
use ds::components::controls::progress::view::ProgressIndicator;
use ds::detail::{Operation, PendingToken};
use ds::prelude::*;
use ds_style::tokens::control_size::ControlSize;

/// The ProgressIndicator section.
#[component]
pub fn ProgressSection() -> Element {
    let mut share = use_signal(|| Fraction(350));
    let mut operation = use_signal(|| Operation::Running(PendingToken::start()));
    rsx! {
        Section { title: "ProgressIndicator", note: "NSProgressIndicator: a determinate value tweens linearly over --t-move to its new value; an unknown amount is a barber pole, a spinner or a turning arc that exists only while an Operation runs and keeps turning under Reduced motion.",
            div { class: "g-row",
                Button { size: ControlSize::Mini, label: "0", onclick: move |_| share.set(Fraction(0)) }
                Button { size: ControlSize::Mini, label: "40", onclick: move |_| share.set(Fraction(400)) }
                Button { size: ControlSize::Mini, label: "100", onclick: move |_| share.set(Fraction(1000)) }
                Button { size: ControlSize::Mini, label: "Start", onclick: move |_| operation.set(Operation::Running(PendingToken::start())) }
                Button { size: ControlSize::Mini, label: "End", onclick: move |_| operation.set(Operation::Idle) }
            }
            for size in [ControlSize::Mini, ControlSize::Small, ControlSize::Regular] {
                div { class: "g-row g-row-top",
                    span { class: "g-name g-type-name", "{size.slug()}" }
                    Specimen { name: "bar".to_owned(),
                        div { style: "width:160px",
                            ProgressIndicator { progress: Progress::Known(share()), size }
                        }
                    }
                    Specimen { name: "bar, running".to_owned(),
                        div { style: "width:160px",
                            ProgressIndicator { progress: Progress::Unknown(operation()), size }
                        }
                    }
                    Specimen { name: "spinner".to_owned(),
                        ProgressIndicator { style: ProgressStyle::Spinner, progress: Progress::Unknown(operation()), size }
                    }
                    Specimen { name: "ring".to_owned(),
                        ProgressIndicator { style: ProgressStyle::Ring, progress: Progress::Known(share()), size }
                    }
                    Specimen { name: "ring, running".to_owned(),
                        ProgressIndicator { style: ProgressStyle::Ring, progress: Progress::Unknown(operation()), size }
                    }
                }
            }
            div { class: "g-row g-row-top",
                Specimen { name: "ring with a glyph".to_owned(),
                    ProgressIndicator { style: ProgressStyle::Ring, progress: Progress::Known(share()), size: ControlSize::Large, glyph: IconSource::Glyph(Icon::Download) }
                }
                Specimen { name: "bar at 0 and 100".to_owned(),
                    div { style: "width:160px; display:flex; flex-direction:column; gap:8px",
                        ProgressIndicator { progress: Progress::Known(Fraction(0)) }
                        ProgressIndicator { progress: Progress::Known(Fraction(1000)) }
                    }
                }
            }
        }
    }
}
