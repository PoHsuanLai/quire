//! IconButtonGroup: small icon buttons on one rounded plate.

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::controls::button_model::{Bezel, ImagePosition};
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;

/// The IconButtonGroup section.
#[component]
pub fn IconGroups() -> Element {
    let icons = [
        ("Bold", Icon::Bold),
        ("Italic", Icon::Italic),
        ("Link", Icon::Link),
    ];
    rsx! {
        Section { title: "IconButtonGroup", note: "IconButtonGroup {{ label, size }}: toolbar-bezel icon buttons that share one rounded plate, as in an editor's block or inline toolbar. Each button keeps its own press and tip.",
            div { class: "g-row",
                for (size , name) in [(ControlSize::Mini, "Mini"), (ControlSize::Small, "Small"), (ControlSize::Regular, "Regular")] {
                    Specimen { name: name.to_string(),
                        IconButtonGroup { label: "Formatting", size,
                            for (label , icon) in icons {
                                Button {
                                    key: "{label}",
                                    bezel: Bezel::Toolbar,
                                    size,
                                    image: ImagePosition::Only,
                                    icon: icon,
                                    label,
                                    onclick: |_| {},
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
