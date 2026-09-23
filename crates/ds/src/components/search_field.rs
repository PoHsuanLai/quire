//! SearchField: a search icon and an inline field heading a list of results, with operator
//! tokens under it (design/04-COMPONENTS.md section 7).

use dioxus::prelude::*;

/// The command menu's and the launcher's search row.
#[component]
pub fn SearchField(
    label: String,
    value: String,
    placeholder: String,
    tokens: Vec<String>,
    oninput: EventHandler<String>,
    onkey: EventHandler<KeyboardData>,
) -> Element {
    todo!()
}
