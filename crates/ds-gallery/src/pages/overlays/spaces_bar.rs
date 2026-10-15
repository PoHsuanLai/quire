//! The Overlays page's Spaces bar: Mission Control's strip of Spaces over the wallpaper, one that
//! follows the pointer (hover it for thumbnails; the plus adds a Space) and one held expanded with
//! a drag over a Space, each with the button that drops it in and lifts it away.

use crate::axes::Axes;
use crate::pages::{Section, Specimen};
use crate::wallpaper;
use dioxus::prelude::*;
use ds::assembly::ds::Inject;
use ds::prelude::*;

/// The Spaces bar section.
#[component]
pub fn SpacesBars() -> Element {
    rsx! {
        Section { title: "SpacesBar", note: "SpacesBar {{ spaces, shown, expansion, on_pick, on_add }}: collapsed it is a row of names with the current one on the pill; the pointer over it expands it to a thumbnail for each Space with its name under it, the current one ringed in the accent. The plus at the right edge adds a Space. A Space a drag is over is lit (DropState::Target), one that could take the drop is outlined (Accepts), and so is the plus (add_drop). Each Space can lie over its desktop's picture (SpaceTile::backdrop), under its windows. It drops in from the top edge and lifts away over --t-overview.",
            // One bar to a row on a wide wall: expanded, three thumbnails are wider than a modal.
            div { class: "g-row g-row-top",
                Bar { name: "SpacesBar, hover it".to_string(), held: None, over: DropState::Idle, add_over: DropState::Idle }
            }
            div { class: "g-row g-row-top",
                Bar { name: "SpacesBar, held expanded, a drag over Desktop 3".to_string(), held: Some(BarExpansion::Expanded), over: DropState::Target, add_over: DropState::Idle }
            }
            div { class: "g-row g-row-top",
                Bar { name: "SpacesBar, held expanded, a drag over the plus".to_string(), held: Some(BarExpansion::Expanded), over: DropState::Idle, add_over: DropState::Target }
            }
        }
    }
}

/// One bar on the wallpaper with its show-and-hide button. `held` is the expansion the specimen
/// fixes, if any; `over` is the third Space's part in a drag and `add_over` the plus's.
#[component]
fn Bar(name: String, held: Option<BarExpansion>, over: DropState, add_over: DropState) -> Element {
    let axes = use_context::<Signal<Axes>>();
    let appearance = {
        let axes = axes.read();
        Appearance {
            theme: Theme::Light,
            accent: axes.accent,
            motion: axes.motion,
            ..Appearance::default()
        }
    };
    let mut shown = use_signal(|| Shown::Visible);
    let mut picked = use_signal(|| 1usize);
    let mut count = use_signal(|| 3usize);
    let tiles: Vec<SpaceTile> = (1..=count())
        .map(|at| {
            SpaceTile::new(BarSpaceKey(at.to_string()), format!("Desktop {at}"))
                .current(Selection::of(&at, &picked()))
                .backdrop(ImageSource(wallpaper::calm_uri(Scheme::Light).to_owned()))
                .in_drag(if at == 3 { over } else { DropState::Idle })
        })
        .collect();
    rsx! {
        Specimen { name, code: Some("spaces, shown, expansion".to_string()),
            div { class: "g-wall g-modal g-modal-wide", style: "background-image:url(\"{wallpaper::uri()}\")",
                Ds { appearance, material: Material::Popover, stylesheet: Inject::Host,
                    div { class: "g-modal-stage",
                        div { class: "g-stage-pad",
                            Button {
                                label: "Show / hide",
                                onclick: move |_| shown.set(match shown() { Shown::Visible => Shown::Hidden, Shown::Hidden => Shown::Visible }),
                            }
                        }
                    }
                    SpacesBar {
                        label: "Spaces",
                        spaces: tiles,
                        shown: shown(),
                        expansion: held,
                        on_pick: move |key: BarSpaceKey| {
                            if let Ok(at) = key.0.parse() {
                                picked.set(at);
                            }
                        },
                        add_drop: add_over,
                        on_add: move |()| count.set(count() + 1),
                    }
                }
            }
        }
    }
}
