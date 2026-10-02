//! Where a sheet hangs (design/30 section 2.5, `NSWindow` sheet): from the top of its window, as a
//! settings sheet drops from the titlebar, from the top of one pane of it, as a document's sheet
//! drops from its own titlebar, or centred, as the shell's app-modal dialog stands.

use crate::host::measure::Anchor;
use ds_core::geometry::units::{Point, Rect};

/// A sheet's attachment.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Attach {
    /// Hanging from the top edge of its window, centred across: it slides down from the titlebar.
    #[default]
    Window,
    /// Hanging from the top edge of one pane (a card, a column), centred across that pane and
    /// as wide as the sheet's width allows within it, not the window: it slides down from the
    /// pane's top edge and is clipped by it. The anchor is the pane's element
    /// (`Anchor::Mounted`) or its rect (`Anchor::Rect`, in the window's client coordinates, which
    /// the caller keeps current if the pane moves; a mounted pane is measured once it has been
    /// laid out).
    Within(Anchor),
    /// Centred both ways, inside a `div.ds-sheet-stage` that fills the overlay and centres it by
    /// flex, not by a transform: the entrance and exit keyframes own the transform, and Blitz's
    /// layout rect leaves a transform out. It still slides in from the top.
    Centre,
    /// Standing `--s-8` above the bottom edge of its root, centred across, never taller than half
    /// of it, and rising from below: Edit Widgets' gallery, which leaves the desktop's top rows,
    /// where a new widget lands, in view. Not in design/30's `Attach {Window, Centre}`: added
    /// where the bottom `Panel` edge was dropped, for the user to settle.
    Bottom,
}

impl Attach {
    /// The `data-attach` value.
    pub fn slug(&self) -> &'static str {
        match self {
            Attach::Window => "window",
            Attach::Within(_) => "within",
            Attach::Centre => "centre",
            Attach::Bottom => "bottom",
        }
    }

    /// The panel as this attachment draws it: bare when it hangs from the window, in a stage when
    /// it is centred or hangs from a pane. `pane` is the pane's rect in the overlay's own
    /// coordinates; a `Within` sheet draws nothing until it is known.
    pub(crate) fn stage(
        &self,
        panel: dioxus::prelude::Element,
        pane: Option<Rect>,
    ) -> dioxus::prelude::Element {
        use dioxus::prelude::*;
        match (self, pane) {
            (Attach::Window | Attach::Bottom, _) => panel,
            (Attach::Centre, _) => rsx! {
                div { class: "ds-sheet-stage", {panel} }
            },
            (Attach::Within(_), Some(pane)) => rsx! {
                div {
                    class: "ds-sheet-stage",
                    "data-attach": "within",
                    style: "{stage_style(pane)}",
                    {panel}
                }
            },
            (Attach::Within(_), None) => rsx! {},
        }
    }

    /// The pane a `Within` sheet hangs from.
    pub(crate) fn pane(&self) -> Option<&Anchor> {
        match self {
            Attach::Within(pane) => Some(pane),
            Attach::Window | Attach::Centre | Attach::Bottom => None,
        }
    }
}

/// The stage's box: the pane's, in the overlay's coordinates.
fn stage_style(pane: Rect) -> String {
    let Rect {
        origin: Point { x, y },
        size,
    } = pane;
    format!(
        "left:{}px;top:{}px;width:{}px;height:{}px",
        x.0, y.0, size.width.0, size.height.0
    )
}
