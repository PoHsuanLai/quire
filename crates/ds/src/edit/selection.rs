//! Selection boxes that land in the frame that draws the text they cover.
//!
//! [`EditHandle::selection_rects`] answers for the last laid-out frame, so a selection drawn from
//! it after the text changed lags by one frame. [`use_selection_rects`] has the host's frame phase
//! publish the boxes between layout and paint instead, as [`use_caret_rect`](crate::edit::caret::use_caret_rect)
//! does for the caret.

use crate::edit::handle::EditHandle;
use crate::host::phase::{Observe, Observed, SelectionWatch, Watch};
use crate::host::position::TextRange;
use dioxus::prelude::*;
use ds_core::geometry::units::Rect;

/// The boxes `range` covers in `handle`'s surface, relative to the surface's border box (draw
/// them at `left`/`top` inside the surface's positioned container), as the surface laid out its
/// text in the frame being drawn. Empty until the surface has mounted and been laid out, while
/// `range` is `None`, collapsed or names nothing in it, and on a host that runs no frame phase
/// (read [`EditHandle::selection_rects`] there).
pub fn use_selection_rects(handle: EditHandle, range: Option<TextRange>) -> ReadSignal<Vec<Rect>> {
    let said = use_hook(|| CopyValue::new(None::<TextRange>));
    // Not a signal: the phase reads it as it runs, ahead of any wake-up a signal would queue.
    let mut held = said;
    held.set(range);
    let rects = use_signal(Vec::<Rect>::new);
    let mut watch = use_signal(|| None::<Watch>);
    let element = handle.element();
    use_effect(move || {
        let Some(surface) = element() else {
            watch.set(None);
            return;
        };
        let watching = SelectionWatch {
            range: said,
            into: rects,
        };
        match handle
            .document()
            .geometry()
            .observe(&surface, Observe::Selection(watching))
        {
            Observed::Watching(found) => watch.set(Some(found)),
            Observed::Unsupported => watch.set(None),
        }
    });
    rects.into()
}
