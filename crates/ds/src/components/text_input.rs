//! TextInput: the one field every text input uses (design/04-COMPONENTS.md section 6).

use crate::components::vocab::Availability;
use dioxus::prelude::*;

/// Boxed for a standalone field, Inline inside another container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputVariant {
    /// `.inp`: bordered, its own focus ring.
    Boxed,
    /// `.inp.inline`: transparent; the container shows focus.
    Inline,
}

/// A single-line text field.
#[component]
pub fn TextInput(
    variant: InputVariant,
    label: String,
    value: String,
    #[props(default)] placeholder: String,
    #[props(default)] availability: Availability,
    oninput: EventHandler<String>,
    #[props(default)] onkey: EventHandler<KeyboardData>,
) -> Element {
    todo!()
}
