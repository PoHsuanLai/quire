//! Kbd: key caps, one `kbd.ds-kbd` per key (design/04-COMPONENTS.md section 9).

use crate::components::vocab::Shortcut;
use dioxus::prelude::*;

/// A key cap's size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum KbdSize {
    /// Data 10.5.
    #[default]
    Regular,
    /// Data 9.5, inside hints.
    Small,
}

/// A shortcut as key caps.
#[component]
pub fn Kbd(shortcut: Shortcut, #[props(default)] size: KbdSize) -> Element {
    todo!()
}
