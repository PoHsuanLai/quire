//! Scrim: dims the card and catches the click that closes (design/04-COMPONENTS.md
//! section 24).

use crate::components::flow::Flow;
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
///
/// `flow` says which layer it dims. [`Flow::Floating`] (the default) is the overlay's scrim:
/// drawn at the end of `.ds` on the scrim layer (`--z-scrim`), above everything the page draws,
/// a layer on the stack that Escape closes. [`Flow::Inline`] (mailo gaps 5) is drawn where the
/// caller renders it, at the caller's stacking level: `position:absolute; inset:0` inside the
/// nearest positioned ancestor, with no overlay, no layer and no Escape of its own, so whatever
/// the caller draws after it in the same container (a peeked reader) sits above it. The
/// layering rule: an inline scrim dims its container's earlier content and lies under its later
/// content and under every floating surface; to put something above it, render it after the
/// scrim in the same positioned container. A press closes it either way (`onclose`), an inline
/// one at once since no other layer can be above it in the overlay's sense.
///
/// The flow is fixed for the scrim's life: key it by the flow to switch.
#[component]
pub fn Scrim(label: String, onclose: EventHandler<()>, #[props(default)] flow: Flow) -> Element {
    let stacking = match flow {
        Flow::Floating => Stacking::Layer(Dismiss::EscOnly),
        Flow::Inline => Stacking::Passive,
    };
    let float = use_float(ZLayer::Scrim, stacking);
    match flow {
        Flow::Floating => {
            float.show(
                scrim_button(&label, move || float.is_top(), onclose),
                onclose,
            );
            rsx! {}
        }
        Flow::Inline => rsx! {
            button {
                r#type: "button",
                class: "ds-scrim",
                "data-flow": flow.attr(),
                "aria-label": "{label}",
                onclick: move |_| onclose.call(()),
            }
        },
    }
}
