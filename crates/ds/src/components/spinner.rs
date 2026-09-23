//! Spinner: activity without a known end, drawn as a ring around its parent
//! (design/04-COMPONENTS.md section 15).

use dioxus::prelude::*;

/// Which activity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpinnerKind {
    /// Work in progress: dashed, spinning.
    Spin,
    /// Idle but live: breathing.
    Breathe,
}

/// An activity ring.
#[component]
pub fn Spinner(kind: SpinnerKind) -> Element {
    todo!()
}
