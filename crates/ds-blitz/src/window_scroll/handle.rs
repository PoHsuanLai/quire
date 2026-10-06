//! The app's handle for scrolling a window's containers from code.

use std::rc::Rc;

use blitz_kit::scroll::cmd::ScrollCmd;
use dioxus::prelude::*;

use super::state::WindowScroll;

/// Scrolls the containers of the window the calling component renders in: `ScrollCmd::To`,
/// `By` and `IntoView` run through the same engine as the wheel and the keys (smooth by the
/// design/11 §11.3.11 rule, or at once), never through Blitz's own `scroll_to`, whose 300 ms
/// animation would fight the engine. A command names its container by element `id`.
#[derive(Clone)]
pub struct ScrollHandle {
    scroll: WindowScroll,
    /// Asks the window for a frame, which is when a command runs.
    wake: Rc<dyn Fn()>,
}

impl std::fmt::Debug for ScrollHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScrollHandle").finish_non_exhaustive()
    }
}

impl ScrollHandle {
    /// The handle over `scroll`, asking for a frame through `wake`.
    pub fn new(scroll: WindowScroll, wake: Rc<dyn Fn()>) -> ScrollHandle {
        ScrollHandle { scroll, wake }
    }

    /// Run `command` at the window's next frame.
    pub fn send(&self, command: ScrollCmd) {
        self.scroll.queue(command);
        (self.wake)();
    }
}

/// The scroll handle of the window the calling component renders in. `None` outside a window
/// `launch` runs and a harness builds (a server render, a snapshot), where nothing scrolls.
pub fn use_scroll_handle() -> Option<ScrollHandle> {
    use_hook(try_consume_context::<ScrollHandle>)
}
