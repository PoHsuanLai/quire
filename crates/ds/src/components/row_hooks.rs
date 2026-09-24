//! The pointer hooks a row hands its caller (mailo gaps 2): the sender's name and the time each
//! open a hover card of their own (design/06-INTERACTIONS.md section 3: `sender` and `time`
//! targets inside a `thread` row), and the row itself starts a drag on a press and opens the
//! thread card on entry. quire draws the parts, so it attaches the caller's handlers to them.

use dioxus::prelude::*;

/// A part's pointer entering and leaving, as the events themselves (a caller reads the point to
/// place its card, or the element to anchor it).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartHooks {
    /// The pointer came over the part. It does not reach the row: a pointer entering a part is
    /// already inside the row, so the row's own `onpointerenter` has fired or will not.
    pub onpointerenter: EventHandler<PointerEvent>,
    /// The pointer left the part (for the row, or anywhere else).
    pub onpointerleave: EventHandler<PointerEvent>,
}

/// The two listeners of `hooks`, or none: a part with no hooks carries no handler.
pub(crate) fn enter(hooks: Option<PartHooks>) -> impl FnMut(PointerEvent) + 'static {
    move |event| {
        if let Some(hooks) = hooks {
            hooks.onpointerenter.call(event);
        }
    }
}

/// The leave listener of `hooks`.
pub(crate) fn leave(hooks: Option<PartHooks>) -> impl FnMut(PointerEvent) + 'static {
    move |event| {
        if let Some(hooks) = hooks {
            hooks.onpointerleave.call(event);
        }
    }
}

/// Call `handler` with `event`, if there is one.
pub(crate) fn relay(handler: Option<EventHandler<PointerEvent>>) -> impl FnMut(PointerEvent) {
    move |event| {
        if let Some(handler) = handler {
            handler.call(event);
        }
    }
}
