//! What an [`EditSurface`](crate::EditSurface) remembers between events, none of it drawn: the
//! composition, the press in progress, the last press (for double clicks), whether it holds the
//! keyboard, its element and its IME registration. Kept in cells, not signals: changing any of it
//! must not re-render the app's content.

use crate::edit::clicks::{Clicks, LastPress, clicks_after};
use crate::edit::composition::Composing;
use crate::edit::host::{HostEdit, ImeListener, Probe};
use crate::edit::pointer::EditFocus;
use crate::geometry::measure::BUSY_ATTEMPTS;
use crate::geometry::{Point, Rect};
use crate::time::{FRAME_SLACK, sleep};
use dioxus::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Instant;

/// Whether the primary button went down on the surface and is still down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Pressing {
    /// No press of ours is under way.
    #[default]
    Up,
    /// A press began on the surface.
    Down,
}

/// The surface's memory.
pub(crate) struct SurfaceState {
    pub(crate) composing: Cell<Composing>,
    pub(crate) pressing: Cell<Pressing>,
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
            pressing: Cell::new(Pressing::Up),
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
        let when = Instant::now();
        let clicks = clicks_after(self.last_press.get(), at, when);
        self.last_press.set(Some(LastPress { at, when, clicks }));
        self.pressing.set(Pressing::Down);
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
    host: HostEdit,
    element: Rc<MountedData>,
    write: impl Fn(&HostEdit, &MountedData) -> Probe<()> + 'static,
) {
    spawn(async move {
        for _ in 0..BUSY_ATTEMPTS {
            match write(&host, &element) {
                Probe::Busy => sleep(FRAME_SLACK).await,
                Probe::Found(()) | Probe::Unknown => return,
            }
        }
    });
}
