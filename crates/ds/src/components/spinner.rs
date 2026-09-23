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

impl SpinnerKind {
    /// The `data-kind` word.
    fn slug(self) -> &'static str {
        match self {
            SpinnerKind::Spin => "spin",
            SpinnerKind::Breathe => "breathe",
        }
    }
}

/// An activity ring, drawn around its positioned parent (`inset:-4px`). A standalone size is
/// not specified (TODO(O-9)).
#[component]
pub fn Spinner(kind: SpinnerKind) -> Element {
    rsx! {
        span { class: "ds-spinner", "data-kind": kind.slug(), "aria-hidden": "true" }
    }
}
