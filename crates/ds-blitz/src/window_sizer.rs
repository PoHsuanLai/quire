//! A window's size from inside it: resize it, and learn who resized it. See
//! [`use_window_sizer`]. The matching of a resize to our own request is `crate::window_fit`.

use crate::window_fit::{SizeLedger, SizeOrigin};
use crate::window_size::Extent;
use dioxus::prelude::*;
use dioxus_native::winit::dpi::LogicalSize;
use dioxus_native::winit::window::Window;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

/// Resizes the window the calling component renders in, and says who last resized it.
///
/// The window's content size is its surface: no frame, in logical pixels. On Wayland the
/// compositor may answer a request with another size (it is tiled, maximized, or the request is
/// below the window's least); that answer reads as [`SizeOrigin::Person`].
#[derive(Clone)]
pub struct WindowSizer {
    window: Arc<dyn Window>,
    ledger: Rc<RefCell<SizeLedger>>,
    origin: Signal<Option<SizeOrigin>>,
}

impl std::fmt::Debug for WindowSizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowSizer").finish_non_exhaustive()
    }
}

impl WindowSizer {
    /// The sizer of `window`.
    pub(crate) fn new(window: Arc<dyn Window>) -> WindowSizer {
        let size = window.surface_size();
        let known = Extent::new(size.width, size.height);
        WindowSizer {
            window,
            ledger: Rc::new(RefCell::new(SizeLedger::new(known))),
            origin: Signal::new(None),
        }
    }

    /// Ask the window to be `size` logical pixels. It resizes when the platform answers: the
    /// new size reaches the content as a resize (and [`origin`](WindowSizer::origin) says
    /// `Requested`), not at once. A window never goes below its least, whatever is asked.
    pub fn request_size(&self, size: Extent) {
        let scale = self.window.scale_factor();
        self.ledger
            .borrow_mut()
            .request(size.physical_at(scale), Instant::now());
        // Where the platform applies it at once it says so; the resize event may still follow.
        let _applied = self
            .window
            .request_surface_size(LogicalSize::new(size.width, size.height).into());
    }

    /// The window's size now, in logical pixels.
    pub fn size(&self) -> Extent {
        let size = self.window.surface_size();
        Extent::new(size.width, size.height)
            .logical_at(self.window.scale_factor())
            .unwrap_or(Extent::new(size.width, size.height))
    }

    /// Who resized the window last: `None` until it has been resized (a window that never is
    /// stays `None`), then [`SizeOrigin::Requested`] for a size this app asked for, and
    /// [`SizeOrigin::Person`] for any other. Reading it in a component subscribes the component
    /// to the next resize.
    pub fn origin(&self) -> Option<SizeOrigin> {
        (self.origin)()
    }

    /// The window was resized to `size` physical pixels: record who did it.
    pub(crate) fn resized(&self, size: Extent) {
        let origin = self.ledger.borrow_mut().resized(size, Instant::now());
        let mut slot = self.origin;
        if let Some(origin) = origin {
            slot.set(Some(origin));
        }
    }
}

/// The sizer of the window the calling component renders in. `None` outside a window `launch`
/// runs and a harness builds (a server render, a snapshot), where nothing resizes.
pub fn use_window_sizer() -> Option<WindowSizer> {
    use_hook(try_consume_context::<WindowSizer>)
}
