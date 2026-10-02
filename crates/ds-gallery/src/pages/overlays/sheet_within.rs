//! Sheet attached to a pane: `Attach::Within` hangs it from the top edge of the pane it is
//! given, centred over that pane, with the window's other pane left alone.

use crate::pages::{Section, Specimen};
use crate::wallpaper;
use dioxus::prelude::*;
use ds::assembly::ds::Inject;
use ds::components::controls::button_model::Answers;
use ds::components::overlays::sheet_attach::Attach;
use ds::host::measure::use_rect;
use ds::prelude::*;

/// The Sheet-within-a-pane section.
#[component]
pub fn SheetWithinSection() -> Element {
    let mut open = use_signal(|| true);
    let card = use_rect();
    let appearance = super::catalogue::appearance(Theme::Light);
    rsx! {
        Section { title: "Sheet: attached to a pane", note: "Attach::Within(anchor) hangs the sheet from the top edge of one pane (a card, a column), centred over it and no wider than min(560, 88%) of it, clipped by the pane as it slides in; the sidebar and the window's other pane stay as they are. The anchor is the pane's element (Anchor::Mounted) or its rect.",
            Specimen { name: "Attach::Within, Regular".to_string(), code: Some("attach: Attach::Within(Anchor::Mounted(card))".to_string()),
                div { class: "g-wall g-modal", style: "background-image:url(\"{wallpaper::uri()}\")",
                    Ds { appearance, material: Material::Sheet, stylesheet: Inject::Host,
                        div { class: "g-modal-stage g-within-stage",
                            div { class: "g-within-side" }
                            div { class: "g-within-card", onmounted: move |event| card.on_mounted(event),
                                div { class: "g-stage-pad",
                                    Button { label: "Show the sheet", onclick: move |_| open.set(true) }
                                }
                            }
                        }
                        if let Some(anchor) = card.anchor() {
                            Sheet {
                                label: "Rules",
                                onclose: move |()| open.set(false),
                                shown: Some(if open() { Shown::Visible } else { Shown::Hidden }),
                                attach: Attach::Within(anchor),
                                div { class: "g-panel",
                                    h3 { "Rules" }
                                    p { class: "g-note", "Hangs from the card, not the window; Escape closes it." }
                                    Button { answers: Answers::Return, label: "Done", onclick: move |_| open.set(false) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
