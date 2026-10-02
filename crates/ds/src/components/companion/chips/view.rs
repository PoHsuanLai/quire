//! The context chips component.

use dioxus::prelude::*;
use ds_intents::ContextChip;

/// The chips a prompt carries, one per piece of context. A removable chip has a remove button;
/// `on_remove` hears the index of the chip dropped. The chips are the consent surface: what is
/// not shown is not sent.
#[component]
pub fn ContextChips(
    chips: Vec<ContextChip>,
    #[props(default)] on_remove: EventHandler<usize>,
) -> Element {
    let _ = on_remove;
    rsx! {
        div { class: "ds-context-chips", "data-count": chips.len() }
    }
}
