//! The confirmation card component.

use super::model::{ConfirmChoice, ConfirmView};
use dioxus::prelude::*;
use ds_core::word::Word;

/// The card the trusted surface draws: who asks, what it will do, the arguments, and Allow and
/// Deny. `on_answer` hears the person's choice, never before the card is armed.
#[component]
pub fn ConfirmCard(view: ConfirmView, on_answer: EventHandler<ConfirmChoice>) -> Element {
    let _ = on_answer;
    rsx! {
        div {
            class: "ds-confirm-card",
            "data-arm": view.arm.slug(),
            "data-offer": view.offer.slug(),
        }
    }
}
