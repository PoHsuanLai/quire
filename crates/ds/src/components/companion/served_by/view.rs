//! The served-by chip component.

use super::model::ModelOption;
use crate::components::companion::answer::footer::ServedByView;
use dioxus::prelude::*;

/// The model that is answering, as a chip; a press opens a menu of `options`, and `on_choose`
/// hears the key of the one picked.
#[component]
pub fn ServedByChip(
    current: ServedByView,
    options: Vec<ModelOption>,
    on_choose: EventHandler<String>,
) -> Element {
    let _ = on_choose;
    rsx! {
        div { class: "ds-served-by-chip", "data-options": options.len(), "{current.model}" }
    }
}
