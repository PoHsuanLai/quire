//! The Edit page: an app's own two paragraphs and a chip inside `EditSurface`, with the caret an
//! app draws placed from the host's caret rect (the surface draws none). Click in the text to
//! move it; the last input is printed. Spelling is on: a misspelt word gets the dotted
//! underline, and a right-click on it opens the suggestions (the page applies none: its text is
//! fixed).

use super::{Section, Specimen};
use dioxus::prelude::*;
use ds::{
    Chip, ChipVariant, EditHandle, EditInput, EditKind, EditPointer, EditSurface, FRAME_SLACK,
    PointerPhase, Probe, Rect, Spell, TextPosition, sleep, use_edit_handle,
};

/// How many frames the caret waits for a layout before giving up.
const PLACE_ATTEMPTS: usize = 8;

/// The edit page.
#[component]
pub fn EditPage() -> Element {
    let handle = use_edit_handle();
    let mut caret = use_signal(|| TextPosition::new("g1", 10));
    let mut drawn = use_signal(|| None::<Rect>);
    let mut last = use_signal(|| "none yet".to_owned());
    use_effect(move || {
        let at = caret();
        spawn(async move {
            if let Some(rect) = place(handle, &at).await {
                drawn.set(Some(rect));
            }
        });
    });
    let at = caret();
    rsx! {
        Section { title: "EditSurface", note: "An app's own paragraphs and an inline chip (an atom) in an edit surface. The accent bar is the page's own caret, drawn from the host's caret rect: the surface draws no caret or selection. Click in the text to move it.",
            Specimen { name: "caret {at.node.0}:{at.offset.0}, last input {last}",
                div { class: "g-edit",
                    EditSurface {
                        handle,
                        label: "Message",
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

/// The caret's rect at `at`, relative to the surface, once the document has laid it out.
async fn place(handle: EditHandle, at: &TextPosition) -> Option<Rect> {
    for _ in 0..PLACE_ATTEMPTS {
        sleep(FRAME_SLACK).await;
        if let (Probe::Found(caret), Probe::Found(bounds)) =
            (handle.caret_rect(at), handle.bounds())
        {
            return Some(Rect {
                origin: ds::Point {
                    x: caret.origin.x - bounds.origin.x,
                    y: caret.origin.y - bounds.origin.y,
                },
                size: caret.size,
            });
        }
    }
    None
}
