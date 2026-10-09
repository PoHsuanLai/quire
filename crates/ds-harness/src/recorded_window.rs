//! The window host the harness gives an app that asks for one ([`WindowHosting::Recorded`]): a
//! `HostWindow` that keeps every title and icon the app set, so a test reads what
//! `use_window_title` and `WindowHost::set_icon` sent. It draws nothing and places nothing:
//! the frame's requests (move, resize, zoom, tile) are ignored and the state is the default.

use ds::window::host::HostWindow;
use ds::window::icon::WindowIcon;
use ds::window::vocab::{ResizeEdge, Support, TileError, WindowState, WindowTile, Zoom};
use std::cell::RefCell;
use std::rc::Rc;

/// Whether the harness gives the app a window host (`ds::prelude::WindowHost`) at the root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WindowHosting {
    /// None: `use_window_host()` is `None`, as in a document no window holds.
    #[default]
    Absent,
    /// One that records the titles and icons set through it (see `Harness::window_titles`).
    Recorded,
}

#[derive(Default)]
struct Calls {
    titles: Vec<String>,
    icons: Vec<WindowIcon>,
}

/// What the app asked of its window, shared between the host the app holds and the harness.
#[derive(Clone, Default)]
pub(crate) struct WindowLog(Rc<RefCell<Calls>>);

impl WindowLog {
    /// Every title set, oldest first.
    pub(crate) fn titles(&self) -> Vec<String> {
        self.0.borrow().titles.clone()
    }

    /// The icon set last, if any.
    pub(crate) fn icon(&self) -> Option<WindowIcon> {
        self.0.borrow().icons.last().cloned()
    }
}

impl HostWindow for WindowLog {
    fn begin_move(&self) {}

    fn begin_resize(&self, _edge: ResizeEdge) {}

    fn zoom(&self, _zoom: Zoom) {}

    fn minimize(&self) {}

    fn close(&self) {}

    fn tile(&self, _tile: WindowTile) -> Result<(), TileError> {
        Err(TileError::Unsupported)
    }

    fn supports(&self, _tile: WindowTile) -> Support {
        Support::No
    }

    fn state(&self) -> WindowState {
        WindowState::default()
    }

    fn set_title(&self, title: &str) {
        self.0.borrow_mut().titles.push(title.to_owned());
    }

    fn set_icon(&self, icon: &WindowIcon) {
        self.0.borrow_mut().icons.push(icon.clone());
    }
}
