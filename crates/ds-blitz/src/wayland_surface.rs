//! The one module allowed `unsafe`: a window's Wayland connection and `wl_surface`, borrowed from
//! raw-window-handle so a protocol request can name the surface of a window that already exists.
//! winit owns both and offers neither as a Rust object; `adopt` is the whole of what quire takes.
#![allow(unsafe_code)]

use raw_window_handle::{DisplayHandle, RawDisplayHandle, RawWindowHandle, WindowHandle};
use std::ffi::c_void;
use std::marker::PhantomData;
use std::ptr::NonNull;
use wayland_backend::client::Backend;
use wayland_backend::client::ObjectId;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Connection, Proxy};
use wayland_protocols::xdg::shell::client::xdg_toplevel::XdgToplevel;

/// Why a window's connection and surface could not be borrowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(crate) enum AdoptError {
    /// The window or the display is not Wayland's.
    #[error("the window is not a Wayland window")]
    ForeignWindow,
    /// libwayland does not know the surface as a `wl_surface`.
    #[error("the window's surface is not a live wl_surface")]
    DeadSurface,
    /// libwayland does not know the pointer as an `xdg_toplevel`.
    #[error("the window's toplevel is not a live xdg_toplevel")]
    DeadToplevel,
}

/// A window's `wl_surface`, and a connection over the display it lives on, both borrowed from
/// the window's handles: they cannot outlive `'a`, and dropping this closes nothing winit owns
/// (only the proxies and the event queue the connection created).
pub(crate) struct Adopted<'a> {
    connection: Connection,
    surface: WlSurface,
    /// What the pointers were borrowed from.
    handles: PhantomData<(DisplayHandle<'a>, WindowHandle<'a>)>,
}

impl std::fmt::Debug for Adopted<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Adopted")
            .field("surface", &self.surface.id())
            .finish()
    }
}

impl Adopted<'_> {
    /// The connection to make requests on, sharing the window's display.
    pub(crate) fn connection(&self) -> &Connection {
        &self.connection
    }

    /// The window's surface.
    pub(crate) fn surface(&self) -> &WlSurface {
        &self.surface
    }

    /// The window's `xdg_toplevel`, from the pointer winit reports for it (`xdg_toplevel` of
    /// winit's `WindowExtWayland`).
    pub(crate) fn toplevel(&self, toplevel: NonNull<c_void>) -> Result<XdgToplevel, AdoptError> {
        // SAFETY: `toplevel` is the `xdg_toplevel*` of the window these handles came from, live
        // for as long as the window, which `Adopted<'a>` keeps borrowed; `from_ptr` checks the
        // proxy's class against `xdg_toplevel` and answers an error, not a bad object, on a
        // mismatch. Dropping the result destroys no protocol object.
        let id = unsafe { ObjectId::from_ptr(XdgToplevel::interface(), toplevel.as_ptr().cast()) }
            .map_err(|_| AdoptError::DeadToplevel)?;
        XdgToplevel::from_id(&self.connection, id).map_err(|_| AdoptError::DeadToplevel)
    }
}

/// The connection and surface behind a Wayland window's handles.
pub(crate) fn adopt<'a>(
    display: DisplayHandle<'a>,
    window: WindowHandle<'a>,
) -> Result<Adopted<'a>, AdoptError> {
    let (RawDisplayHandle::Wayland(display), RawWindowHandle::Wayland(window)) =
        (display.as_raw(), window.as_raw())
    else {
        return Err(AdoptError::ForeignWindow);
    };
    // SAFETY: `display` is the `wl_display*` of a live connection: raw-window-handle's contract
    // for the `DisplayHandle<'a>` it came from, which `Adopted<'a>` keeps borrowed for as long
    // as the backend exists. A foreign display is never disconnected by the backend: dropping it
    // destroys only the proxies and the event queue created through it.
    let backend = unsafe { Backend::from_foreign_display(display.display.as_ptr().cast()) };
    let connection = Connection::from_backend(backend);
    // SAFETY: `window.surface` is a `wl_surface*` of that display, live for the
    // `WindowHandle<'a>` it came from, which `Adopted<'a>` keeps borrowed; `from_ptr` checks
    // the proxy's class against `wl_surface` and answers an error, not a bad object, on a
    // mismatch.
    let id = unsafe { ObjectId::from_ptr(WlSurface::interface(), window.surface.as_ptr().cast()) }
        .map_err(|_| AdoptError::DeadSurface)?;
    let surface = WlSurface::from_id(&connection, id).map_err(|_| AdoptError::DeadSurface)?;
    Ok(Adopted {
        connection,
        surface,
        handles: PhantomData,
    })
}

/// A `HasDisplayHandle` and `HasWindowHandle` over a test connection and one of its surfaces:
/// what a test hands `adopt` in place of a winit window.
#[cfg(test)]
pub(crate) struct Foreign {
    display: RawDisplayHandle,
    window: RawWindowHandle,
}

#[cfg(test)]
impl Foreign {
    /// The handles of `window`, an object of `connection`: a surface for a real window. `None`
    /// without the system libwayland.
    pub(crate) fn of(connection: &Connection, window: &impl Proxy) -> Option<Foreign> {
        use raw_window_handle::{WaylandDisplayHandle, WaylandWindowHandle};
        use std::ptr::NonNull;
        let display = NonNull::new(connection.backend().display_ptr().cast())?;
        let window = NonNull::new(window.id().as_ptr().cast())?;
        Some(Foreign {
            display: RawDisplayHandle::Wayland(WaylandDisplayHandle::new(display)),
            window: RawWindowHandle::Wayland(WaylandWindowHandle::new(window)),
        })
    }
}

#[cfg(test)]
impl Foreign {
    /// The handles of an X11 window.
    pub(crate) fn x11() -> Foreign {
        use raw_window_handle::{XlibDisplayHandle, XlibWindowHandle};
        Foreign {
            display: RawDisplayHandle::Xlib(XlibDisplayHandle::new(None, 0)),
            window: RawWindowHandle::Xlib(XlibWindowHandle::new(1)),
        }
    }
}

#[cfg(test)]
impl raw_window_handle::HasDisplayHandle for Foreign {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, raw_window_handle::HandleError> {
        // SAFETY: the test keeps the connection and the surface alive for as long as this value.
        Ok(unsafe { DisplayHandle::borrow_raw(self.display) })
    }
}

#[cfg(test)]
impl raw_window_handle::HasWindowHandle for Foreign {
    fn window_handle(&self) -> Result<WindowHandle<'_>, raw_window_handle::HandleError> {
        // SAFETY: as for the display handle.
        Ok(unsafe { WindowHandle::borrow_raw(self.window) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

    #[test]
    fn an_x11_window_is_not_adopted() {
        let window = Foreign::x11();
        let adopted = adopt(
            window.display_handle().expect("a display handle"),
            window.window_handle().expect("a window handle"),
        );
        assert_eq!(adopted.err(), Some(AdoptError::ForeignWindow));
    }
}
