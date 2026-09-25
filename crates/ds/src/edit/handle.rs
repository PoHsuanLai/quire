//! The app's handle on its surface: the geometry reads it needs to draw its own caret and
//! selection, and to place its `/` and `@` menus at the caret.

use crate::edit::host::{HostEdit, Probe};
use crate::edit::position::{TextPosition, TextRange};
use crate::focus::focus_soon;
use crate::geometry::{HostMeasure, Measured, Point, Rect};
use dioxus::prelude::*;
use std::rc::Rc;

/// A handle on one [`EditSurface`](crate::EditSurface): pass it as the surface's `handle`, then
/// read geometry from a handler or a task. Each read answers [`Probe::Busy`] while the renderer
/// holds the document (ask again next frame) and [`Probe::Unknown`] before the surface mounts,
/// with no host, or where nothing addressable is. Geometry is the last layout's: after a change
/// to the text, read on the next frame.
#[derive(Debug, Clone, Copy)]
pub struct EditHandle {
    element: Signal<Option<Rc<MountedData>>>,
    /// The surface's own focus and blur, set as it mounts: a programmatic focus does what a
    /// press does.
    hooks: Signal<Option<SurfaceHooks>>,
    /// The host, read once where the handle is made, so a read needs no scope of its own.
    host: Option<HostEdit>,
    /// The host's rect read, for [`EditHandle::bounds`].
    measure: Option<HostMeasure>,
}

/// What the surface does to take or give up the keyboard: the same as a press or a blur.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SurfaceHooks {
    /// Focus the surface, then tell it (`on_focus` In, the IME on, the IME's target).
    pub(crate) focus: EventHandler<()>,
    /// Take the keyboard away, then tell it (`on_focus` Out, the IME off).
    pub(crate) blur: EventHandler<()>,
}

/// The same handle is the same surface.
impl PartialEq for EditHandle {
    fn eq(&self, other: &Self) -> bool {
        self.element == other.element
    }
}

/// A handle owned by the calling component.
pub fn use_edit_handle() -> EditHandle {
    EditHandle {
        element: use_signal(|| None),
        hooks: use_signal(|| None),
        host: use_hook(try_consume_context::<HostEdit>),
        measure: use_hook(try_consume_context::<HostMeasure>),
    }
}

impl EditHandle {
    /// The surface's element, once mounted; the surface sets it.
    pub(crate) fn set(&self, element: Rc<MountedData>, hooks: SurfaceHooks) {
        let mut slot = self.element;
        slot.set(Some(element));
        let mut surface = self.hooks;
        surface.set(Some(hooks));
    }

    /// The text position under `at`.
    pub fn hit_test(&self, at: Point) -> Probe<TextPosition> {
        self.with(|host, element| (host.hit_test)(element, at))
    }

    /// The caret's box at `position`: `--caret-w` wide from the insertion point, its line's
    /// height.
    pub fn caret_rect(&self, position: &TextPosition) -> Probe<Rect> {
        self.with(|host, element| (host.caret_rect)(element, position))
    }

    /// The boxes `range` covers: one per line of text, one per whole atom.
    pub fn selection_rects(&self, range: &TextRange) -> Probe<Vec<Rect>> {
        self.with(|host, element| (host.selection_rects)(element, range))
    }

    /// The surface's own border box, in the same coordinates as the other reads: an app draws
    /// its caret layer inside the surface's container at `caret_rect - bounds`.
    pub fn bounds(&self) -> Probe<Rect> {
        let Some(HostMeasure(measure)) = self.measure else {
            return Probe::Unknown;
        };
        match self
            .element
            .try_peek()
            .ok()
            .and_then(|element| element.clone())
        {
            Some(element) => match measure(&element) {
                Measured::At(rect) => Probe::Found(rect),
                Measured::Busy => Probe::Busy,
                Measured::Unknown => Probe::Unknown,
            },
            None => Probe::Unknown,
        }
    }

    /// Give the surface the keyboard, a frame later if the document is busy, exactly as a press
    /// does: `on_focus` hears [`EditFocus::In`](crate::EditFocus), the IME is switched on and
    /// the surface becomes its target.
    pub fn focus(&self) {
        match self.hooks.try_peek().ok().and_then(|hooks| *hooks) {
            Some(hooks) => hooks.focus.call(()),
            None => {
                if let Some(element) = self.element.try_peek().ok().and_then(|e| e.clone()) {
                    focus_soon(element);
                }
            }
        }
    }

    /// Take the keyboard away from the surface: `on_focus` hears
    /// [`EditFocus::Out`](crate::EditFocus), an open composition ends and the IME is switched
    /// off, as at a real blur.
    pub fn blur(&self) {
        if let Some(hooks) = self.hooks.try_peek().ok().and_then(|hooks| *hooks) {
            hooks.blur.call(());
        }
    }

    fn with<T>(&self, read: impl FnOnce(&HostEdit, &MountedData) -> Probe<T>) -> Probe<T> {
        let Some(host) = self.host else {
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
