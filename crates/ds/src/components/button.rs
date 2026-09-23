//! Button: a labelled action in five variants (design/04-COMPONENTS.md section 1).
//! Markup: `button.ds-button[data-variant]`, `aria-pressed` only for a toggle Mini.

use crate::components::vocab::{Availability, Switch};
use crate::icon::Icon;
use dioxus::prelude::*;

/// Which button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonVariant {
    /// Accent fill: Send, Compose.
    Primary,
    /// Surface-2 with a line: the ghost button.
    Secondary,
    /// Small, surface-2: Reply, RSVP, hover-card actions.
    Mini,
    /// Text only: the quiet link-button.
    Quiet,
    /// As Mini at rest; red only on hover.
    Danger,
}

/// A labelled action.
#[component]
pub fn Button(
    variant: ButtonVariant,
    label: String,
    #[props(default)] icon: Option<Icon>,
    #[props(default)] pressed: Option<Switch>,
    #[props(default)] availability: Availability,
    onclick: EventHandler<()>,
) -> Element {
    todo!()
}
