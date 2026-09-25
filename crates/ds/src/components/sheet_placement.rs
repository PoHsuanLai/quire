//! Where a sheet stands in its root (sill Q91): at the top, as a settings sheet drops from the
//! title, or centred, as macOS centres its shutdown dialog.

use dioxus::prelude::*;

/// A sheet's place in its root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SheetPlacement {
    /// 36 px from the top, centred across (the sheet as it was).
    #[default]
    Top,
    /// Centred both ways. Written `data-placement="centre"`, inside a `div.ds-sheet-stage` that
    /// fills the overlay and centres it by flex, not by a transform: the entrance and exit
    /// keyframes own the transform, and Blitz's layout rect leaves a transform out.
    Centre,
}

impl SheetPlacement {
    /// The `data-placement` value: only a centred sheet carries one, so a top sheet's markup is
    /// what it was.
    pub(crate) fn attribute(self) -> Option<&'static str> {
        match self {
            SheetPlacement::Top => None,
            SheetPlacement::Centre => Some("centre"),
        }
    }

    /// The panel as this placement draws it: bare at the top, in the centring stage otherwise.
    pub(crate) fn stage(self, panel: Element) -> Element {
        match self {
            SheetPlacement::Top => panel,
            SheetPlacement::Centre => rsx! {
                div { class: "ds-sheet-stage", {panel} }
            },
        }
    }
}
