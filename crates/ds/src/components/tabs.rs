//! Tabs: switch between pages of one surface, with a spring underline
//! (design/04-COMPONENTS.md section 12).

use dioxus::prelude::*;

/// A tab bar.
#[component]
pub fn Tabs<T: Clone + PartialEq + 'static>(
    label: String,
    tabs: Vec<(T, String)>,
    value: T,
    onchange: EventHandler<T>,
) -> Element {
    todo!()
}
