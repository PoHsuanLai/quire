//! `quire_window_v1` on a window that exists: the app tells the compositor that it should draw the
//! window's shadow and clip its corners, at the radius the app's own window edge has
//! (`set_frame(compositor_shadow, radius)`). The object is made on the app's own connection
//! (`crate::wayland_surface`) for the window's `xdg_toplevel`, which winit reports, and it is
//! kept for as long as the window: the compositor forgets what an object said when it is
//! destroyed, so dropping this destroys it and nothing sooner. A compositor without the global
//! is `FrameError::Unsupported`, and the window is drawn as it always was.

use crate::wayland_surface::{AdoptError, Adopted};
use ds::style::tokens::shape::Radius;
use proto::client::quire_window_manager_v1::QuireWindowManagerV1;
use proto::client::quire_window_v1::{FrameMode, QuireWindowV1};
use std::ffi::c_void;
use std::ptr::NonNull;
use wayland_client::globals::{BindError, GlobalError, GlobalListContents, registry_queue_init};
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::{Connection, Dispatch, EventQueue, Proxy, QueueHandle};

mod proto;

/// Why the compositor could not be told.
#[derive(Debug, thiserror::Error)]
pub(crate) enum FrameError {
    /// The window's toplevel could not be borrowed.
    #[error("adopting the toplevel: {0}")]
    Adopt(#[from] AdoptError),
    /// The registry could not be read.
    #[error("reading the registry: {0}")]
    Registry(#[from] GlobalError),
    /// The compositor offers no `quire_window_manager_v1`.
    #[error("the compositor offers no quire_window_manager_v1: {0}")]
    Unsupported(#[from] BindError),
    /// The connection failed while the request went out.
    #[error("sending the request: {0}")]
    Send(#[from] wayland_client::backend::WaylandError),
}

/// The corner radius a window's edge is drawn with, in logical pixels: `--r-window`, the token the
/// stylesheet reads. A titlebar-only window's `--r-window-plain` is the same number, so one value
/// serves every window; if a look gives it its own, this takes the window's kind.
pub(crate) fn window_radius() -> u32 {
    u32::from(Radius::Window.px())
}

/// One window's `quire_window_v1`, alive until this is dropped.
pub(crate) struct QuireWindow {
    object: QuireWindowV1,
    /// The queue the object's events arrive on; nothing reads them, so they are dropped unread
    /// whenever a request goes out.
    queue: EventQueue<Quiet>,
    connection: Connection,
    /// The radius last sent.
    sent: Option<u32>,
}

impl std::fmt::Debug for QuireWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuireWindow")
            .field("object", &self.object.id())
            .field("sent", &self.sent)
            .finish()
    }
}

/// Make the window's `quire_window_v1` from `toplevel`, its `xdg_toplevel` as winit reports it.
pub(crate) fn attach(
    adopted: &Adopted<'_>,
    toplevel: NonNull<c_void>,
) -> Result<QuireWindow, FrameError> {
    let toplevel = adopted.toplevel(toplevel)?;
    let connection = adopted.connection().clone();
    let (globals, queue) = registry_queue_init::<Quiet>(&connection)?;
    let manager: QuireWindowManagerV1 = globals.bind(&queue.handle(), 1..=1, ())?;
    let object = manager.get_window(&toplevel, &queue.handle(), ());
    // Objects already created stay valid, and no `quit` is heard at version 1.
    manager.destroy();
    Ok(QuireWindow {
        object,
        queue,
        connection,
        sent: None,
    })
}

impl QuireWindow {
    /// Say the compositor draws the shadow and clips the corners at `radius`: double-buffered, so
    /// it applies at the window's next commit. A radius already sent is not sent again.
    pub(crate) fn set_radius(&mut self, radius: u32) -> Result<(), FrameError> {
        if self.sent == Some(radius) {
            return Ok(());
        }
        self.object.set_frame(FrameMode::CompositorShadow, radius);
        self.connection.flush()?;
        self.sent = Some(radius);
        // The emphasis and placement events nothing here reads.
        let _ = self.queue.dispatch_pending(&mut Quiet);
        Ok(())
    }
}

impl Drop for QuireWindow {
    fn drop(&mut self) {
        self.object.destroy();
        let _ = self.connection.flush();
    }
}

/// The state of the queue: it asks, and what comes back is not read.
struct Quiet;

impl Dispatch<WlRegistry, GlobalListContents> for Quiet {
    fn event(
        _: &mut Self,
        _: &WlRegistry,
        _: <WlRegistry as Proxy>::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<QuireWindowManagerV1, ()> for Quiet {
    fn event(
        _: &mut Self,
        _: &QuireWindowManagerV1,
        _: <QuireWindowManagerV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<QuireWindowV1, ()> for Quiet {
    fn event(
        _: &mut Self,
        _: &QuireWindowV1,
        _: <QuireWindowV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

#[cfg(test)]
mod fake_compositor;
#[cfg(test)]
mod tests;
