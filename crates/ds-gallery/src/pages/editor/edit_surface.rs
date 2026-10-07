//! The Edit page: an app's own two paragraphs and a chip inside `EditSurface`, with the caret an
//! app draws placed from the caret box the host publishes with each frame (the surface draws
//! none). Click in the text to move it; the last input is printed. Spelling is on: a misspelt word gets the dotted
//! underline, and a right-click on it opens the suggestions (the page applies none: its text is
//! fixed).

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::controls::chip::{Chip, ChipVariant};
use ds::edit::caret::use_caret_rect;
use ds::edit::handle::use_edit_handle;
use ds::edit::input::EditInput;
use ds::edit::pointer::EditPointer;
use ds::host::captured::PointerPhase;
use ds::host::position::{EditKind, TextPosition};
use ds::prelude::*;
use ds::root::common::Common;
use ds::spell::lang::Spell;

/// The edit page.
#[component]
pub fn EditPage() -> Element {
    let handle = use_edit_handle();
    let mut caret = use_signal(|| TextPosition::new("g1", 10));
    let mut last = use_signal(|| "none yet".to_owned());
    let drawn = use_caret_rect(handle, Some(caret()));
    let at = caret();
    rsx! {
        Section { title: "EditSurface", note: "An app's own paragraphs and an inline chip (an atom) in an edit surface. The accent bar is the page's own caret, drawn from the caret box the host publishes in the frame that draws the text: the surface draws no caret or selection. Click in the text to move it.",
            Specimen { name: "caret {at.node.0}:{at.offset.0}, last input {last}",
                div { class: "g-edit",
                    EditSurface {
                        common: Common { aria_label: Some("Message".to_string()), ..Common::default() },
                        handle,
                        spell: Spell::On { lang: None },
                        caret: Some(caret()),
                        on_input: move |input: EditInput| last.set(format!("{input:?}")),
                        on_pointer: move |pointer: EditPointer| {
                            if pointer.phase == PointerPhase::Press
                                && let Some(position) = pointer.position
                            {
                                caret.set(position);
                            }
                        },
                        div { class: "g-edit-body",
                            p { "data-edit-node": "g0", "Dear Ada, a quick noet about speling:" }
                            p { "data-edit-node": "g1",
                                "Lunch with "
                                span { class: "g-edit-chip", "data-edit-node": "g2", "data-edit-kind": EditKind::Atom.slug(),
                                    Chip { variant: ChipVariant::Token, text: "Grace Hopper" }
                                }
                                " on Friday? The bar is placed from the rect the host reports."
                            }
                        }
                    }
                    if let Some(rect) = drawn() {
                        div { class: "g-edit-caret",
                            style: "left:{rect.origin.x.0}px; top:{rect.origin.y.0}px; height:{rect.size.height.0}px",
                        }
                    }
                }
            }
        }
    }
}
