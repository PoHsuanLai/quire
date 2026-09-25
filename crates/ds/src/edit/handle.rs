//! The app's handle on its surface: the geometry reads it needs to draw its own caret and
//! selection, and to place its `/` and `@` menus at the caret.

use crate::edit::host::{HostEdit, Probe};
use crate::edit::position::{TextPosition, TextRange};
use crate::focus::focus_soon;
use crate::geometry::{Point, Rect};
use dioxus::prelude::*;
use std::rc::Rc;

/// A handle on one [`EditSurface`](crate::EditSurface): pass it as the surface's `handle`, then
/// read geometry from a handler or a task. Each read answers [`Probe::Busy`] while the renderer
/// holds the document (ask again next frame) and [`Probe::Unknown`] before the surface mounts,
/// with no host, or where nothing addressable is. Geometry is the last layout's: after a change
/// to the text, read on the next frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditHandle {
    element: Signal<Option<Rc<MountedData>>>,
}

/// A handle owned by the calling component.
pub fn use_edit_handle() -> EditHandle {
    EditHandle {
        element: use_signal(|| None),
    }
}

impl EditHandle {
    /// The surface's element, once mounted; the surface sets it.
    pub(crate) fn set(&self, element: Rc<MountedData>) {
        let mut slot = self.element;
        slot.set(Some(element));
    }

    /// The text position under `at`.
    pub fn hit_test(&self, at: Point) -> Probe<TextPosition> {
        self.with(|host, element| (host.hit_test)(element, at))
    }

    /// The caret's box at `position`: zero width, its line's height.
    pub fn caret_rect(&self, position: &TextPosition) -> Probe<Rect> {
        self.with(|host, element| (host.caret_rect)(element, position))
    }

    /// The boxes `range` covers: one per line of text, one per whole atom.
    pub fn selection_rects(&self, range: &TextRange) -> Probe<Vec<Rect>> {
        self.with(|host, element| (host.selection_rects)(element, range))
    }

    /// Give the surface the keyboard, a frame later if the document is busy.
    pub fn focus(&self) {
        if let Some(element) = self.element.peek().clone() {
            focus_soon(element);
        }
    }

    fn with<T>(&self, read: impl FnOnce(&HostEdit, &MountedData) -> Probe<T>) -> Probe<T> {
        let Some(host) = try_consume_context::<HostEdit>() else {
            return Probe::Unknown;
        };
        match self
            .element
            .try_peek()
            .ok()
            .and_then(|element| element.clone())
        {
            Some(element) => read(&host, &element),
            None => Probe::Unknown,
        }
    }
}
