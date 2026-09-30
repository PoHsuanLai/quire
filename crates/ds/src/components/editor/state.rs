//! What an [`EditSurface`](crate::EditSurface) remembers between events, none of it drawn: the
//! composition, the press in progress, the last press (for double clicks), whether it holds the
//! keyboard, its element and its IME registration. Kept in cells, not signals: changing any of it
//! must not re-render the app's content.

use crate::edit::clicks::{Clicks, LastPress, clicks_after};
use crate::edit::composition::Composing;
use crate::edit::pointer::EditFocus;
use crate::host::document::DocumentHost;
use crate::host::ime::ImeListener;
use crate::host::measure::BUSY_ATTEMPTS;
use crate::host::parts::EditHost;
use crate::host::probe::Probe;
use dioxus::prelude::*;
use ds_core::geometry::units::{Point, Rect};
use ds_core::time::{FRAME_SLACK, clock::sleep};
use ds_core::vocab::PressPhase;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// Whether the host routes the pointer to the surface (a press captured it, until release).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Capture {
    /// The surface's own pointer events serve.
    #[default]
    Free,
    /// The host's capture carries moves and the release.
    Held,
}

/// The surface's memory.
pub(crate) struct SurfaceState {
    pub(crate) composing: Cell<Composing>,
    pub(crate) pressing: Cell<PressPhase>,
    pub(crate) capture: Cell<Capture>,
    last_press: Cell<Option<LastPress>>,
    pub(crate) focus: Cell<EditFocus>,
    pub(crate) element: RefCell<Option<Rc<MountedData>>>,
    pub(crate) listener: Cell<Option<ImeListener>>,
    /// The IME cursor area the app asked for last.
    pub(crate) ime_area: Cell<Option<Rect>>,
}

impl Default for SurfaceState {
    fn default() -> Self {
        SurfaceState {
            composing: Cell::new(Composing::Idle),
            pressing: Cell::new(PressPhase::Idle),
            capture: Cell::new(Capture::Free),
            last_press: Cell::new(None),
            focus: Cell::new(EditFocus::Out),
            element: RefCell::new(None),
            listener: Cell::new(None),
            ime_area: Cell::new(None),
        }
    }
}

impl SurfaceState {
    /// Count a press at `at` now, remembering it for the next.
    pub(crate) fn press(&self, at: Point) -> Clicks {
        let when = ds_core::time::clock::now();
        let clicks = clicks_after(self.last_press.get(), at, when);
        self.last_press.set(Some(LastPress { at, when, clicks }));
        self.pressing.set(PressPhase::Pressed);
        self.capture.set(Capture::Free);
        clicks
    }

    /// The count of the last press, for the drag and release that follow it.
    pub(crate) fn last_clicks(&self) -> Clicks {
        self.last_press
            .get()
            .map_or(Clicks(1), |press| press.clicks)
    }

    /// The mounted element, if any.
    pub(crate) fn element(&self) -> Option<Rc<MountedData>> {
        self.element.borrow().clone()
    }
}

/// Run a host write against `element` from a task of the calling scope, a frame later whenever
/// the document is busy (the same wait `ds::focus_soon` makes).
pub(crate) fn write_soon(
    host: Rc<dyn DocumentHost>,
    element: Rc<MountedData>,
    write: impl Fn(&dyn EditHost, &MountedData) -> Probe<()> + 'static,
) {
    spawn(async move {
        for _ in 0..BUSY_ATTEMPTS {
            let Some(edit) = host.edit() else {
                return;
            };
            match write(edit, &element) {
                Probe::Busy => sleep(FRAME_SLACK).await,
                Probe::Found(()) | Probe::Unknown => return,
            }
        }
    });
}
