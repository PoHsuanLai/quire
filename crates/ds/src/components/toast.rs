//! Toast: the undo toast, one dark pill with a pull tab (design/04-COMPONENTS.md section 23).

use crate::overlay::toast_hub::ToastHub;
use dioxus::prelude::*;

/// The enclosing `Ds`'s toast manager: `push(text, undo)`, one visible at a time.
pub fn use_toasts() -> ToastHub {
    todo!()
}

/// Renders the toast. `Ds` places it; consumers never do.
#[component]
pub fn ToastHost() -> Element {
    todo!()
}
