//! Where a sheet hangs (design/30 section 2.5, `NSWindow` sheet): from the top of its window, as a
//! settings sheet drops from the titlebar, or centred, as the shell's app-modal dialog stands.

use ds_core::word::Word;

/// A sheet's attachment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum Attach {
    /// Hanging from the top edge of its window, centred across: it slides down from the titlebar.
    #[default]
    Window,
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
    /// The panel as this attachment draws it: bare when it hangs, in the centring stage
    /// otherwise.
    pub(crate) fn stage(self, panel: dioxus::prelude::Element) -> dioxus::prelude::Element {
        use dioxus::prelude::*;
        match self {
            Attach::Window | Attach::Bottom => panel,
            Attach::Centre => rsx! {
                div { class: "ds-sheet-stage", {panel} }
            },
        }
    }
}
