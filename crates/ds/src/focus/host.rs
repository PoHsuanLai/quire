//! Moving keyboard focus into a mounted element, the one focus write the design system does.
//!
//! dioxus-native-dom's `set_focus` borrows its document mutably *when it is called*. A task
//! woken in the same turn as a dirty scope is polled inside `render_immediate`, while the
//! mutation writer holds that borrow, so a focus change made from such a task panicked
//! ("RefCell already borrowed", sill FINDINGS Q43: a palette's field focusing on mount while
//! its results re-rendered). Every focus change goes through [`focus_soon`]: the host's
//! [`HostFocus`] answers [`Focused::Busy`] instead, and with no host the call is guarded
//! (`crate::guarded`) so the collision is `Busy` too; the change is tried again a frame later.

use crate::geometry::measure::BUSY_ATTEMPTS;
use crate::guarded::guarded_call;
use crate::time::{FRAME_SLACK, sleep};
use dioxus::prelude::*;
use std::rc::Rc;

/// One attempt at moving the focus through the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Focused {
    /// The element has the focus.
    Done,
    /// The document is busy (rendering); try again next frame.
    Busy,
    /// The host cannot focus this element (not its node, or gone).
    Unknown,
}

/// The host's own focus write, provided as root context by `ds-native` (`ds_native::launch`,
/// its harness, and `ds_native::focus::provide` for any other Blitz host); without one, focus
/// goes through `MountedData::set_focus`, guarded so a renderer that holds its document answers
/// [`Focused::Busy`] instead of panicking.
#[derive(Debug, Clone, Copy)]
pub struct HostFocus(pub fn(&MountedData) -> Focused);

/// Give `element` the keyboard from a task of the calling scope, a frame later whenever the
/// document is busy. Best-effort: a renderer without focus still shows the element, and the
/// person can click into it. Call it from a handler or a hook, never from inside a task that
/// the renderer may be polling with its document held (it spawns, it does not focus).
pub(crate) fn focus_soon(element: Rc<MountedData>) {
    spawn(async move {
        let _ = focus_element(&element).await;
    });
}

/// Move the focus to `element`, waiting out a busy document for up to `BUSY_ATTEMPTS` frames.
pub(crate) async fn focus_element(element: &MountedData) -> Focused {
    let host = try_consume_context::<HostFocus>();
    for _ in 0..BUSY_ATTEMPTS {
        let tried = match host {
            Some(HostFocus(focus)) => focus(element),
            None => unhosted(element).await,
        };
        match tried {
            Focused::Busy => sleep(FRAME_SLACK).await,
            Focused::Done | Focused::Unknown => return tried,
        }
    }
    Focused::Busy
}

/// A focus change with no host seam: the call and its future guarded, a panic read as busy.
async fn unhosted(element: &MountedData) -> Focused {
    match guarded_call(|| element.set_focus(true)).await {
        Some(Ok(())) => Focused::Done,
        Some(Err(_)) => Focused::Unknown,
        None => Focused::Busy,
    }
}
