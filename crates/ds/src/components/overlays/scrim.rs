//! The scrim button a peek draws under its card (design/04-COMPONENTS.md section 24): it dims
//! the card behind and closes on a click. Crate-private: a sheet dims nothing on macOS, so the
//! public `Scrim` component and its strengths are gone (design/30 Part 4); the peek is the one
//! modal that still dims.

use dioxus::prelude::*;

/// The scrim button itself, for a modal that draws its own (`Peek`): a click closes when
/// `closes()` says the modal is the topmost layer.
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
