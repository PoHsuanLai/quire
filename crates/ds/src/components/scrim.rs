//! Scrim: dims the card and catches the click that closes (design/04-COMPONENTS.md
//! section 24).

use crate::components::popover::{Dismiss, Stacking, use_float};
use crate::tokens::ZLayer;
use dioxus::prelude::*;

/// The scrim button itself, for a modal that draws its own (`Peek`, `Sheet`): a click closes
/// when `closes()` says the modal is the topmost layer.
pub(crate) fn scrim_button(
    label: &str,
    closes: impl Fn() -> bool + 'static,
    onclose: EventHandler<()>,
) -> Element {
    rsx! {
        button {
            r#type: "button",
            class: "ds-scrim",
            "aria-label": "{label}",
            onclick: move |_| {
                if closes() {
                    onclose.call(());
                }
            },
        }
    }
}

/// A dimming layer that closes on click.
#[component]
pub fn Scrim(label: String, onclose: EventHandler<()>) -> Element {
    let float = use_float(ZLayer::Scrim, Stacking::Layer(Dismiss::EscOnly));
    float.show(
        scrim_button(&label, move || float.is_top(), onclose),
        onclose,
    );
    rsx! {}
}
