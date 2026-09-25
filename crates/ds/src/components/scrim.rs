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
/// `layer` (mailo gaps 6) gives an inline scrim a stacking layer of its own, written as
/// `z-index: var(--z-…)` on it. Without one it sets no z-index, so a positioned row the caller
/// draws after it in the same container (a list row that is `position:relative` for its hover
/// strip) paints over it. The caller picks the layer: above its own rows (`ZLayer::Raise` over
/// rows that set none) and below its floating surfaces (the reader it peeks, a menu), which
/// then need a layer above the one chosen. A floating scrim is always on `--z-scrim` and ignores
/// it.
///
/// The flow is fixed for the scrim's life: key it by the flow to switch.
#[component]
pub fn Scrim(
    label: String,
    onclose: EventHandler<()>,
    #[props(default)] flow: Flow,
    #[props(default)] layer: Option<ZLayer>,
) -> Element {
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
                style: layer.map(layer_style),
                "aria-label": "{label}",
                onclick: move |_| onclose.call(()),
            }
        },
    }
}

/// An inline scrim's own layer, as its `style`: `z-index:var(--z-raise)`.
fn layer_style(layer: ZLayer) -> String {
    format!("z-index:var({})", layer.var().as_str())
}

#[cfg(test)]
mod tests {
    use super::layer_style;
    use crate::tokens::ZLayer;

    #[test]
    fn a_layer_is_written_as_its_token() {
        assert_eq!(layer_style(ZLayer::Raise), "z-index:var(--z-raise)");
        assert_eq!(layer_style(ZLayer::LinkPill), "z-index:var(--z-link-pill)");
    }
}
