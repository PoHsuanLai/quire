//! Telling a menu's caller where its highlight is or should go: under a
//! caller's cursor the pointer only asks, and under the menu's own the caller hears each move
//! after the render that made it. Split from `menu`; `cursor` holds the pure rules.

use crate::components::menus::menu::cursor::MenuCursor;
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// Under a caller's cursor, the pointer over another choice asks for it.
pub(crate) fn asks(
    active: MenuCursor,
    pointed: Option<usize>,
    on_active: Option<EventHandler<Option<usize>>>,
) {
    if let (MenuCursor::Controlled(shown), Some(index), Some(on_active)) =
        (active, pointed, on_active)
        && shown != Some(index)
    {
        on_active.call(Some(index));
    }
}

/// What `follow_active` last told the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Reported {
    /// Nothing yet.
    #[default]
    Never,
    /// That no choice is highlighted.
    Cleared,
    /// That this choice is.
    At(usize),
}

impl From<Option<usize>> for Reported {
    fn from(current: Option<usize>) -> Self {
        current.map_or(Reported::Cleared, Reported::At)
    }
}

/// Under the menu's own cursor, tell the caller after this render when the highlight moved.
pub(crate) fn follow_active(
    active: MenuCursor,
    current: Option<usize>,
    reported: CopyValue<Reported>,
    on_active: Option<EventHandler<Option<usize>>>,
) {
    let (MenuCursor::Auto, Some(on_active)) = (active, on_active) else {
        return;
    };
    let mut reported = reported;
    if *reported.peek() == Reported::from(current) {
        return;
    }
    reported.set(Reported::from(current));
    queue_effect(move || on_active.call(current));
}
