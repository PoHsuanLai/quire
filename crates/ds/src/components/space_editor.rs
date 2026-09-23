//! SpaceEditor: the field, dots, stops, grain, presets and contrast checks, plus the Space dots
//! that switch Spaces (design/04-COMPONENTS.md section 32, design/21-SPACES.md section 6).
//! Every colour it shows comes from `space::palette`; it computes none.

use crate::appearance::Scheme;
use crate::components::vocab::{Here, Shortcut};
use crate::space::{FrameVars, SpaceLook};
use dioxus::prelude::*;

/// Which of a Space's dots is being edited: 0, 1 or 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct DotIndex(pub u8);

/// The Space editor panel.
#[component]
pub fn SpaceEditor(
    look: SpaceLook,
    scheme: Scheme,
    active_dot: DotIndex,
    onchange: EventHandler<SpaceLook>,
) -> Element {
    todo!()
}

/// One Space's dot in the sidebar foot.
#[component]
pub fn SpaceDot(
    name: String,
    frame: FrameVars,
    here: Here,
    shortcut: Shortcut,
    onclick: EventHandler<()>,
) -> Element {
    todo!()
}
