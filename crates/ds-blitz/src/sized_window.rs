//! The window a `WindowSizer` asks to resize: winit's (`WinitSized`), or a headless host's
//! stand-in (the harness's fake), which answers as a test says.

use crate::screen_area::ScreenArea;
use crate::window_requests::Requests;
use crate::window_screen::window_area;
use crate::window_size::Extent;
use crate::window_sizer::WindowSizer;
use dioxus_native::winit::dpi::LogicalSize;
use dioxus_native::winit::window::Window;
use std::sync::Arc;

/// What a [`WindowSizer`] needs of a window.
pub trait SizedWindow {
    /// The window's content size now, in physical pixels.
    fn surface_size(&self) -> Extent;

    /// Physical pixels per logical pixel for this window.
    fn scale_factor(&self) -> f64;

    /// Ask for a content size of `logical` pixels. The platform answers when it likes, or never;
    /// an answer is a resize that the window's host reports to `answer_to`
    /// (`WindowSizer::resized`, in physical pixels).
    fn request_surface_size(&self, logical: Extent, answer_to: &WindowSizer);

    /// The output the window is on, if the platform lists one.
    fn screen(&self) -> Option<ScreenArea>;
}

/// A winit window, whose resizes arrive as `WindowEvent::SurfaceResized`, except one the
/// platform applies at once and answers only in `request_surface_size`'s return (Wayland): that
/// is handed to the event loop (`requests`), which delivers the event the platform did not send.
pub(crate) struct WinitSized {
    pub(crate) window: Arc<dyn Window>,
    pub(crate) requests: Option<Requests>,
}

impl SizedWindow for WinitSized {
    fn surface_size(&self) -> Extent {
        let size = self.window.surface_size();
        Extent::new(size.width, size.height)
    }

    fn scale_factor(&self) -> f64 {
        self.window.scale_factor()
    }

    fn request_surface_size(&self, logical: Extent, _answer_to: &WindowSizer) {
        // Where the platform applies it at once it says so and may send no resize event.
        let applied = self
            .window
            .request_surface_size(LogicalSize::new(logical.width, logical.height).into());
        if let Some(requests) = &self.requests {
            requests.applied(self.window.id(), applied);
        }
    }

    fn screen(&self) -> Option<ScreenArea> {
        window_area(self.window.as_ref())
    }
}
