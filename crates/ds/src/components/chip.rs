//! Chip: a small label that states a fact (design/04-COMPONENTS.md section 10).

use crate::components::avatar::AvatarFace;
use crate::components::vocab::PulseKey;
use crate::space::Verdict;
use crate::tokens::LabelHue;
use dioxus::prelude::*;

/// Which chip.
#[derive(Debug, Clone, PartialEq)]
pub enum ChipVariant {
    /// Accent-soft: every label chip in the Spaces prototype.
    Accent,
    /// One Candy hue per named label.
    Label(LabelHue),
    /// Surface-2 with a line.
    Neutral,
    /// A recipient, with an avatar and a remove button.
    Person(AvatarFace),
    /// A search operator in the command menu.
    Token,
    /// A contrast check's pass or fail.
    Status(Verdict),
}

/// A small label.
#[component]
pub fn Chip(
    variant: ChipVariant,
    text: String,
    #[props(default)] onremove: Option<EventHandler<()>>,
    #[props(default)] pulse: Option<PulseKey>,
) -> Element {
    todo!()
}
