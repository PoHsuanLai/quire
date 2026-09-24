//! Telling a menu's caller where its highlight is or should go (mailo gaps 2): under a
//! caller's cursor the pointer only asks, and under the menu's own the caller hears each move
//! after the render that made it. Split from `menu`; `menu_cursor` holds the pure rules.

use crate::components::menu_cursor::Cursor;
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// Under a caller's cursor, the pointer over another choice asks for it.
pub(crate) fn asks(
    active: Cursor,
    pointed: Option<usize>,
    on_active: Option<EventHandler<Option<usize>>>,
) {
    if let (Cursor::Controlled(shown), Some(index), Some(on_active)) = (active, pointed, on_active)
        && shown != Some(index)
    {
        on_active.call(Some(index));
    }
}

/// Under the menu's own cursor, tell the caller after this render when the highlight moved.
pub(crate) fn follow_active(
    active: Cursor,
    current: Option<usize>,
    reported: CopyValue<Option<Option<usize>>>,
    on_active: Option<EventHandler<Option<usize>>>,
) {
    let (Cursor::Auto, Some(on_active)) = (active, on_active) else {
        return;
    };
    let mut reported = reported;
    if *reported.peek() == Some(current) {
        return;
    }
    reported.set(Some(current));
    queue_effect(move || on_active.call(current));
}
