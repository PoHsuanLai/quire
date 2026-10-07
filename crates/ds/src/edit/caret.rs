//! A caret box that lands in the frame that draws the text it follows.
//!
//! An app that reads [`EditHandle::caret_rect`] after its text changed reads the layout of the
//! frame before: the glyph is drawn in frame N and the caret a frame later. [`use_caret_rect`]
//! asks the host's frame phase to publish the box instead; the window loop runs the phase
//! between laying the new text out and painting it when the caret moved, so the box is in the
//! signal, and in the document, when frame N paints.

use crate::edit::handle::EditHandle;
use crate::host::phase::{CaretWatch, Observe, Observed, Watch};
use crate::host::position::TextPosition;
use dioxus::prelude::*;
use ds_core::geometry::units::Rect;

/// The caret's box at `at` in `handle`'s surface, relative to the surface's border box (draw it
/// at `left`/`top` inside the surface's positioned container), as the surface laid out its text
/// in the frame being drawn. `None` until the surface has mounted and been laid out, while `at`
/// is `None` or names nothing in it, and on a host that runs no frame phase (read
/// [`EditHandle::caret_rect`] there).
pub fn use_caret_rect(handle: EditHandle, at: Option<TextPosition>) -> ReadSignal<Option<Rect>> {
    let position = use_hook(|| CopyValue::new(None::<TextPosition>));
    // Not a signal: the phase reads it as it runs, ahead of any wake-up a signal would queue.
    let mut said = position;
    said.set(at);
    let rect = use_signal(|| None::<Rect>);
    let mut watch = use_signal(|| None::<Watch>);
    let element = handle.element();
    use_effect(move || {
        let Some(surface) = element() else {
            watch.set(None);
            return;
        };
        let watching = CaretWatch {
            at: position,
            into: rect,
        };
        match handle
            .document()
            .geometry()
            .observe(&surface, Observe::Caret(watching))
        {
            Observed::Watching(found) => watch.set(Some(found)),
            Observed::Unsupported => watch.set(None),
        }
    });
    rect.into()
}
