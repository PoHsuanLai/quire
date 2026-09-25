//! Sheet: the general modal panel for settings and dialogs, derived from Peek
//! (design/04-COMPONENTS.md section 24): Peek Center's surface, over a scrim, sized by its
//! content up to Peek Center's inset (O-14's proposal).

use crate::components::popover::{Dismiss, Stacking, escape_closes, use_float};
use crate::components::scrim::scrim_button;
use crate::motion::anim::Anim;
use crate::motion::entrance::use_entrance;
use crate::tokens::ZLayer;
use dioxus::prelude::*;

/// A modal panel.
#[component]
pub fn Sheet(label: String, onclose: EventHandler<()>, children: Element) -> Element {
    let float = use_float(ZLayer::Peek, Stacking::Layer(Dismiss::EscOnly));
    let presence = use_entrance(Anim::PeekIn);
    let close = format!("Close {label}");
    float.show(
        rsx! {
            {scrim_button(&close, move || float.is_top(), onclose)}
            div {
                class: "ds-sheet",
                "data-presence": presence.slug(),
                role: "dialog",
                "aria-label": "{label}",
                onkeydown: move |event| escape_closes(float, &event, onclose),
                {children}
            }
        },
        onclose,
    );
    rsx! {}
}
