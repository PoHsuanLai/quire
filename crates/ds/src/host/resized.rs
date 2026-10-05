//! That the window's size or scale changed: a counter the host bumps, so a hook that measured an
//! element can measure it again. Layout follows the event by a frame, so a reader waits that
//! frame before it reads (the read `layout::use_layout` falls back to when the host runs no frame
//! phase does; with a phase, the host publishes the rect after the layout that followed).

use dioxus::prelude::*;

/// How many times the window has changed size or scale, as a signal: provided as root context by
/// the host that sees the window's events (`launch`), and absent where nothing resizes (a server
/// render, a shell surface whose host measures for itself).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowResized(Signal<u64>);

impl WindowResized {
    /// A counter at zero, in the calling component's scope.
    pub fn new() -> Self {
        WindowResized(Signal::new(0))
    }

    /// The window changed: whoever reads [`WindowResized::count`] is told.
    pub fn bump(&self) {
        let mut count = self.0;
        let next = count.peek().wrapping_add(1);
        count.set(next);
    }

    /// How many changes there have been; reading it subscribes the caller.
    pub fn count(&self) -> u64 {
        (self.0)()
    }
}

impl Default for WindowResized {
    fn default() -> Self {
        WindowResized::new()
    }
}
