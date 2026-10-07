//! A window's size from inside it: resize it, learn who resized it, and whether our own request
//! was answered. See [`use_window_sizer`]. The matching of a resize to a request is
//! `crate::size_ledger`; the window it asks is `crate::sized_window`.

use crate::screen_area::ScreenArea;
use crate::size_ledger::{ANSWER_WINDOW, Expiry, Serial, SizeLedger, SizeOrigin, SizeRequest};
use crate::sized_window::{SizedWindow, WinitSized};
use crate::window_size::Extent;
use dioxus::core::spawn_forever;
use dioxus::prelude::*;
use dioxus_native::winit::window::Window;
use ds::base::time::clock::{now, sleep};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// Resizes the window the calling component renders in, and says who last resized it.
///
/// The window's content size is its surface: no frame, in logical pixels. On Wayland the
/// compositor may answer a request with another size (it is tiled, maximized, or the request is
/// below the window's least); that answer reads as [`SizeOrigin::Person`].
#[derive(Clone)]
pub struct WindowSizer {
    window: Rc<dyn SizedWindow>,
    ledger: Rc<RefCell<SizeLedger>>,
    origin: Signal<Option<SizeOrigin>>,
    request: Signal<SizeRequest>,
}

impl std::fmt::Debug for WindowSizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowSizer").finish_non_exhaustive()
    }
}

impl WindowSizer {
    /// The sizer of the winit `window`.
    pub(crate) fn new(window: Arc<dyn Window>) -> WindowSizer {
        WindowSizer::over(Rc::new(WinitSized(window)))
    }

    /// The sizer of `window`, for a host that is not a winit window (the headless harness). The
    /// host reports each resize to [`WindowSizer::resized`]. Call it inside a Dioxus runtime (in a
    /// component, or `VirtualDom::in_runtime`): its state is signals of the root scope.
    pub fn over(window: Rc<dyn SizedWindow>) -> WindowSizer {
        let known = window.surface_size();
        WindowSizer {
            window,
            ledger: Rc::new(RefCell::new(SizeLedger::new(known))),
            origin: Signal::new_in_scope(None, ScopeId::ROOT),
            request: Signal::new_in_scope(SizeRequest::Idle, ScopeId::ROOT),
        }
    }

    /// Ask the window to be `size` logical pixels. It resizes when the platform answers: the
    /// new size reaches the content as a resize (and [`origin`](WindowSizer::origin) says
    /// `Requested`), not at once. A window never goes below its least, whatever is asked.
    /// [`request`](WindowSizer::request) follows the request: `Pending` now, `Idle` once a
    /// resize answers it, `Expired` if none does within half a second. Call it inside the
    /// window's Dioxus runtime (an event handler, an effect): the expiry is a task of the root.
    pub fn request_size(&self, size: Extent) {
        let scale = self.window.scale_factor();
        let serial = self
            .ledger
            .borrow_mut()
            .request(size.physical_at(scale), now());
        self.settle_request(SizeRequest::Pending(size));
        let sizer = self.clone();
        spawn_forever(async move {
            sleep(ANSWER_WINDOW).await;
            sizer.expired(serial, size);
        });
        self.window.request_surface_size(size, self);
    }

    /// The window's size now, in logical pixels.
    pub fn size(&self) -> Extent {
        let size = self.window.surface_size();
        size.logical_at(self.window.scale_factor()).unwrap_or(size)
    }

    /// The output the window is on, with the area a window may use and its scale; `None` where
    /// the platform lists no output. See [`ScreenArea`] for what a client can know. A window
    /// that has not been mapped yet has no output of its own, so this is the output it is likeliest to open on.
    pub fn screen(&self) -> Option<ScreenArea> {
        self.window.screen()
    }

    /// Who resized the window last: `None` until it has been resized (a window that never is
    /// stays `None`), then [`SizeOrigin::Requested`] for a size this app asked for, and
    /// [`SizeOrigin::Person`] for any other. Reading it in a component subscribes the component
    /// to the next resize. A request that expires changes nothing here: the origin is what it was
    /// before the request, and [`request`](WindowSizer::request) says the request lapsed.
    pub fn origin(&self) -> Option<SizeOrigin> {
        (self.origin)()
    }

    /// What became of this app's last request: [`SizeRequest::Idle`] before any and once a resize
    /// settled it, [`SizeRequest::Pending`] while it waits for an answer, and
    /// [`SizeRequest::Expired`] when none came in half a second. Reading it in a component
    /// subscribes the component to the change.
    pub fn request(&self) -> SizeRequest {
        (self.request)()
    }

    /// The window was resized to `size` physical pixels: record who did it.
    pub fn resized(&self, size: Extent) {
        let origin = self.ledger.borrow_mut().resized(size, now());
        let mut slot = self.origin;
        if let Some(origin) = origin {
            slot.set(Some(origin));
            self.settle_request(SizeRequest::Idle);
        }
    }

    /// The answer window of request `serial`, for `asked`, has passed.
    fn expired(&self, serial: Serial, asked: Extent) {
        if self.ledger.borrow_mut().expire(serial) == Expiry::Lapsed {
            self.settle_request(SizeRequest::Expired(asked));
        }
    }

    /// Show `next` as the request's state, waking readers only when it changed.
    fn settle_request(&self, next: SizeRequest) {
        let mut slot = self.request;
        if *slot.peek() != next {
            slot.set(next);
        }
    }
}

/// The sizer of the window the calling component renders in. `None` outside a window `launch`
/// runs and a harness builds (a server render, a snapshot), where nothing resizes.
pub fn use_window_sizer() -> Option<WindowSizer> {
    use_hook(try_consume_context::<WindowSizer>)
}
