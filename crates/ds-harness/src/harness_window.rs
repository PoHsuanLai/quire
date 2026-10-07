//! The harness's window, as a test sees it: what a component asked of it, what size it is, and
//! the person resizing it. Its answers to a request are `HarnessConfig::with_sizer_ack`.

use crate::harness::Harness;
use crate::snapshot::Viewport;
use ds_blitz::{Extent, WindowSizer};

impl Harness {
    /// The sizes (logical px) components have asked the window for with `request_size`, oldest
    /// first, answered or not.
    pub fn window_requests(&self) -> Vec<Extent> {
        self.doc.window.requests()
    }

    /// The window's size now, logical px: the viewport until a request is answered or the
    /// person resizes it.
    pub fn window_size(&self) -> Extent {
        self.doc.sizer.size()
    }

    /// The person drags the window to `size` (logical px), which reaches the app as a resize no
    /// request asked for (`SizeOrigin::Person`). The document is laid out again at the new size,
    /// at the same device scale, so a test can check what a narrower or wider window does.
    pub fn resize_window(&mut self, size: Extent) {
        let viewport = Viewport {
            width: size.width,
            height: size.height,
            ..self.viewport
        };
        self.doc.resize(viewport);
        self.viewport = viewport;
        let window = self.doc.window.clone();
        let sizer = self.doc.sizer.clone();
        self.within(|| window.resize(size, &sizer));
        self.settle_now();
    }

    /// The sizer components read with `use_window_sizer`, for a test that reads its state
    /// without a component.
    pub fn window_sizer(&self) -> WindowSizer {
        self.doc.sizer.clone()
    }
}
