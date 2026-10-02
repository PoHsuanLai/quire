//! Following the pointer past an element's own box: [`use_pointer_capture`] is the hook for a
//! drag that must not stop where the element does (panning a picture, scrubbing a timeline,
//! resizing a divider). Blitz delivers a move only to the element under the pointer and has no
//! pointer capture of its own (FINDINGS "Events"), so a press asks the host to route every move
//! and the primary release to the element's sink until the button comes up, the way an edit
//! surface does for a selection drag.
//!
//! ```ignore
//! let hold = use_pointer_capture(move |pointer: CapturedPointer| match pointer.phase {
//!     PointerPhase::Drag => pan_to(pointer.at),
//!     PointerPhase::Release => done(),
//!     PointerPhase::Press => {}
//! });
//! rsx! {
//!     div {
//!         onmounted: move |event| hold.on_mounted(event),
//!         onpointerdown: move |event| {
//!             if hold.begin() == PointerHold::Local { /* own onpointermove / onpointerup serve */ }
//!         },
//!     }
//! }
//! ```
//!
//! Positions are the window's logical pixels, as [`CapturedPointer`] says.

use crate::host::captured::CapturedPointer;
use crate::host::document::{DocumentHost, use_document_host};
use crate::host::probe::Probe;
use dioxus::prelude::*;
use std::rc::Rc;

/// Who follows the pointer after a press.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerHold {
    /// The host routes every move and the release to the sink, wherever the pointer is.
    Captured,
    /// The host cannot (no host, the element is not mounted yet, the document is busy): the
    /// element's own `pointermove` and `pointerup` are all there is, and a drag that leaves its
    /// box is noticed at the next move.
    Local,
}

/// An element that can capture the pointer. Copy: signals only.
#[derive(Clone, Copy)]
pub struct PointerCapture {
    element: Signal<Option<Rc<MountedData>>>,
    host: CopyValue<Rc<dyn DocumentHost>>,
    sink: Callback<CapturedPointer>,
}

impl std::fmt::Debug for PointerCapture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PointerCapture")
    }
}

/// Capture handle for the element that calls [`PointerCapture::on_mounted`]; `on_pointer` hears
/// each captured move and the release.
pub fn use_pointer_capture(on_pointer: impl FnMut(CapturedPointer) + 'static) -> PointerCapture {
    PointerCapture {
        element: use_signal(|| None),
        host: use_hook(|| CopyValue::new(use_document_host())),
        sink: use_callback(on_pointer),
    }
}

impl PointerCapture {
    /// Remember the element a press will capture the pointer for: wire it to the element's
    /// `onmounted`.
    pub fn on_mounted(&self, event: MountedEvent) {
        let mut element = self.element;
        element.set(Some(event.data()));
    }

    /// Ask the host to follow the pointer for this element until the primary button comes up:
    /// call it from the element's `onpointerdown`.
    pub fn begin(&self) -> PointerHold {
        let Some(element) = self.element.peek().clone() else {
            return PointerHold::Local;
        };
        let host = self.host.read().clone();
        let held = host.edit().map(|edit| edit.capture(&element, self.sink));
        match held {
            Some(Probe::Found(())) => PointerHold::Captured,
            Some(Probe::Busy | Probe::Unknown) | None => PointerHold::Local,
        }
    }
}
