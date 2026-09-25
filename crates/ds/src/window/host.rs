//! The host seam a window frame drives (FINDINGS "Window frame"). `ds` stays renderer-free: it
//! names what a client-decorated window asks of its platform, and each host fills it in —
//! ds-native over the winit window `launch` opens (`ds_native::window`), sill over shell-host's
//! `SurfaceHandle` (`begin_move`, `begin_resize(edge)`, `set_maximized`, `use_toplevel_state`).
//! A trait, not a table of `fn`s like `HostEdit`: each host holds its own window handle, and a
//! test's stub records what it was asked.
//!
//! The host is provided as [`WindowHost`] context, which also carries the last [`WindowState`]
//! as a signal, so the frame redraws when the window is zoomed or loses the keyboard. The host
//! pushes a change with [`WindowHost::refresh`] (re-read its own `state`) or
//! [`WindowHost::publish`] (a state it was told, as an xdg configure).

use super::vocab::{ResizeEdge, Support, TileError, WindowState, WindowTile, Zoom};
use dioxus::prelude::*;
use std::rc::Rc;

/// What a client-decorated window asks of its platform. Every request is fire-and-forget: the
/// compositor answers with a configure (a new size, a new state), which the host reports through
/// [`WindowHost::refresh`] or [`WindowHost::publish`].
pub trait HostWindow {
    /// Start an interactive move with the pointer button that is down now. Call it while the
    /// button is still held: Wayland's `xdg_toplevel.move` needs the press's serial.
    fn begin_move(&self);
    /// Start an interactive resize from `edge` with the pointer button that is down now.
    fn begin_resize(&self, edge: ResizeEdge);
    /// Maximize, restore or toggle between the two.
    fn zoom(&self, zoom: Zoom);
    /// Minimize (a Wayland client cannot un-minimize itself).
    fn minimize(&self);
    /// Close the window, as the compositor's close would.
    fn close(&self);
    /// Place the window: [`WindowTile::Fill`] maximizes; the others need a client that can
    /// place itself, and answer [`TileError::Unsupported`] where it cannot.
    fn tile(&self, tile: WindowTile) -> Result<(), TileError>;
    /// Whether [`HostWindow::tile`] would place the window there, asked before the menu offers it.
    fn supports(&self, tile: WindowTile) -> Support;
    /// The window's state now.
    fn state(&self) -> WindowState;
}

/// The host's window, as root context: the seam and its last reported state.
#[derive(Clone)]
pub struct WindowHost {
    host: Rc<dyn HostWindow>,
    state: Signal<WindowState>,
}

impl std::fmt::Debug for WindowHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowHost")
            .field("state", &*self.state.peek())
            .finish_non_exhaustive()
    }
}

impl WindowHost {
    /// Wrap `host`, reading its state once. Call it inside a component (the state is a signal
    /// the calling scope owns); [`use_window_host_provider`] does and provides it.
    pub fn new(host: Rc<dyn HostWindow>) -> Self {
        let state = Signal::new(host.state());
        WindowHost { host, state }
    }

    /// The seam.
    pub fn host(&self) -> &dyn HostWindow {
        self.host.as_ref()
    }

    /// The last reported state; a component that reads it redraws when it changes.
    pub fn state(&self) -> WindowState {
        (self.state)()
    }

    /// Re-read the host's own state, after a window event that may have changed it (a resize, a
    /// focus change).
    pub fn refresh(&self) {
        self.publish(self.host.state());
    }

    /// Report `state`, as a host told it by its compositor does. Writes only a change.
    pub fn publish(&self, state: WindowState) {
        let mut signal = self.state;
        if *signal.peek() != state {
            signal.set(state);
        }
    }
}

/// Provide `host` to the calling component's subtree, once. ds-native's `launch` does this for
/// its winit window; sill does it for a shell-host surface; a test does it with a stub.
pub fn use_window_host_provider(host: impl FnOnce() -> Rc<dyn HostWindow>) -> WindowHost {
    use_context_provider(|| WindowHost::new(host()))
}

/// The enclosing host's window, if there is one (none in a webview, an SSR render or a shell
/// surface that has not provided one).
pub fn use_window_host() -> Option<WindowHost> {
    try_use_context::<WindowHost>()
}

/// The window's state, redrawn on every change the host reports; the default (its own size,
/// not fullscreen, active) with no host.
pub fn use_window_state() -> WindowState {
    use_window_host().map_or(WindowState::default(), |host| host.state())
}
