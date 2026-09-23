//! IconButton: an icon-only action, `aria-label` mandatory (design/04-COMPONENTS.md section 2).

use crate::components::vocab::{Availability, Switch};
use crate::icon::Icon;
use dioxus::prelude::*;

/// Which icon button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconButtonVariant {
    /// Reader and composer tools, 28 x 26.
    Tool,
    /// Sidebar foot, 24 x 24, on the frame.
    Foot,
    /// A row's hover strip, 26 x 26 round.
    Strip,
    /// A square tile on the frame (account tiles).
    Pin,
}

/// An icon-only action.
#[component]
pub fn IconButton(
    variant: IconButtonVariant,
    icon: Icon,
    label: String,
    #[props(default)] tooltip: Option<String>,
    #[props(default)] pressed: Option<Switch>,
    #[props(default)] expanded: Option<Switch>,
    #[props(default)] availability: Availability,
    onclick: EventHandler<()>,
) -> Element {
    todo!()
}
