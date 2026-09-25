//! The window seam (`ds::HostWindow`) over the winit window `launch` opens (FINDINGS "Window
//! frame"). Moves, resizes, zoom and minimize are winit's own requests; close goes through
//! blitz-shell's `request_window_close`, which drops the window and ends the event loop as the
//! compositor's close does. The left half, right half and centre placements need a client that
//! can read and set its own position: winit answers `outer_position` with `NotSupported` on
//! Wayland and ignores `set_outer_position` there, so they are `Support::No` on Wayland and work
//! on X11 (and wherever winit can place a window).
//!
//! `launch` provides it as `ds::WindowHost` and refreshes the state on every resize and focus
//! change. Another Blitz host (shell-host's surfaces, through sill) provides its own
//! `HostWindow` over its `SurfaceHandle` instead.

use crate::window_place::{Area, placement};
use blitz_traits::shell::ShellProvider;
use dioxus_native::winit::dpi::{PhysicalPosition, PhysicalSize};
use dioxus_native::winit::window::{ResizeDirection, Window};
use ds::{
    Activation, Fullscreen, HostWindow, Maximized, ResizeEdge, Support, TileError, WindowState,
    WindowTile, Zoom,
};
use std::sync::Arc;

/// The winit window, as the frame's host.
pub struct WinitWindow {
    window: Arc<dyn Window>,
    shell: Arc<dyn ShellProvider>,
}

impl std::fmt::Debug for WinitWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WinitWindow")
            .field("window", &self.window.id())
            .finish_non_exhaustive()
    }
}

impl WinitWindow {
    /// The seam over `window`, closing through `shell` (the document's shell provider).
    pub fn new(window: Arc<dyn Window>, shell: Arc<dyn ShellProvider>) -> Self {
        WinitWindow { window, shell }
    }

    /// The output the window is on, if winit can say where it is.
    fn output(&self) -> Option<Area> {
        let monitor = self.window.current_monitor()?;
        let at = monitor.position()?;
        let size = monitor.current_video_mode()?.size();
        Some(Area {
            x: at.x,
            y: at.y,
            width: size.width,
            height: size.height,
        })
    }

    /// Whether the client can place itself: it can read its own position and its output's.
    fn placeable(&self) -> Support {
        match (self.window.outer_position(), self.output()) {
            (Ok(_), Some(_)) => Support::Yes,
            _ => Support::No,
        }
    }
}

impl HostWindow for WinitWindow {
    fn begin_move(&self) {
        // Refused where the platform has no interactive move (a web canvas, a phone), or when
        // no button is down: the pointer stays where it was, which is all the frame can do.
        let _ = self.window.drag_window();
    }

    fn begin_resize(&self, edge: ResizeEdge) {
        // Refused on macOS and where there is no interactive resize: nothing happens.
        let _ = self.window.drag_resize_window(direction(edge));
    }

    fn zoom(&self, zoom: Zoom) {
        let maximized = match zoom {
            Zoom::Toggle => !self.window.is_maximized(),
            Zoom::Maximize => true,
            Zoom::Restore => false,
        };
        self.window.set_maximized(maximized);
    }

    fn minimize(&self) {
        self.window.set_minimized(true);
    }

    fn close(&self) {
        self.shell.request_window_close();
    }

    fn tile(&self, tile: WindowTile) -> Result<(), TileError> {
        if tile == WindowTile::Fill {
            self.window.set_maximized(true);
            return Ok(());
        }
        let output = match (self.placeable(), self.output()) {
            (Support::Yes, Some(output)) => output,
            _ => return Err(TileError::Unsupported),
        };
        let size = self.window.outer_size();
        let Some(area) = placement(output, (size.width, size.height), tile) else {
            return Err(TileError::Unsupported);
        };
        if self.window.is_maximized() {
            self.window.set_maximized(false);
        }
        self.window
            .set_outer_position(PhysicalPosition::new(area.x, area.y).into());
        let _ = self
            .window
            .request_surface_size(PhysicalSize::new(area.width, area.height).into());
        Ok(())
    }

    fn supports(&self, tile: WindowTile) -> Support {
        match tile {
            WindowTile::Fill => Support::Yes,
            WindowTile::LeftHalf | WindowTile::RightHalf | WindowTile::Centre => self.placeable(),
        }
    }

    fn state(&self) -> WindowState {
        WindowState {
            maximized: match self.window.is_maximized() {
                true => Maximized::On,
                false => Maximized::Off,
            },
            fullscreen: match self.window.fullscreen() {
                Some(_) => Fullscreen::On,
                None => Fullscreen::Off,
            },
            activated: match self.window.has_focus() {
                true => Activation::Active,
                false => Activation::Inactive,
            },
        }
    }
}

/// winit's name for an edge.
fn direction(edge: ResizeEdge) -> ResizeDirection {
    match edge {
        ResizeEdge::Top => ResizeDirection::North,
        ResizeEdge::Bottom => ResizeDirection::South,
        ResizeEdge::Left => ResizeDirection::West,
        ResizeEdge::Right => ResizeDirection::East,
        ResizeEdge::TopLeft => ResizeDirection::NorthWest,
        ResizeEdge::TopRight => ResizeDirection::NorthEast,
        ResizeEdge::BottomLeft => ResizeDirection::SouthWest,
        ResizeEdge::BottomRight => ResizeDirection::SouthEast,
    }
}

/// Who draws the window's frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Decorations {
    /// The compositor or the toolkit's fallback (winit's default): an app that draws no
    /// `WindowFrame` of its own.
    #[default]
    Server,
    /// The app: its root draws `ds::WindowFrame::Titlebar`, and the window asks for no
    /// decorations.
    Client,
}

impl Decorations {
    /// winit's decorations flag for it.
    pub(crate) fn winit(self) -> bool {
        match self {
            Decorations::Server => true,
            Decorations::Client => false,
        }
    }
}
