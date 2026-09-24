//! What a pick does to the menu (design/06-INTERACTIONS.md section 5): a floating menu closes
//! and then yields the value, but a toggle checklist (C's view properties, the row's label
//! picker, mailo's Labels and page Properties) toggles and stays open (`C:1762-1767`,
//! `C:1987-2002`). Split from `menu`, with the menu's closing and gesture states.

use crate::components::flow::Flow;
use crate::components::menu_cursor::Cursor;
use crate::components::menu_tracker::Tracker;
use dioxus::prelude::*;

/// Whether picking a choice closes the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PickDismiss {
    /// A pick calls `onpick`, then `onclose` (the default).
    #[default]
    Close,
    /// A pick calls `onpick` only: the menu stays open with the cursor where it was, so the
    /// caller toggles the choice's check and the next pick follows. Escape and an outside
    /// press still close it.
    Stay,
}

/// Where a pointer gesture over an open menu is: a press-drag-release onto an item picks it
/// (design/13 section 13.3.2), which is a release arriving after the pointer came in with no
/// press of its own inside the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Gesture {
    /// The pointer has not been over the menu.
    Outside,
    /// The pointer came in; no press started inside the menu.
    Entered,
    /// A press started inside the menu: its release is a click, not a drag's end.
    Pressed,
}

/// Whether the menu is closing, and how.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Closing {
    /// Open.
    No,
    /// Something was picked and `onclose` has run: nothing else picks (Blitz follows a
    /// press-drag-release with a click on the same item).
    Picked,
    /// Playing `menu-out`; `onclose` runs when it settles.
    Fading,
}

/// The handler a panel calls with a picked value: under [`PickDismiss::Close`] the first pick
/// closes the menu and every later one is ignored; under [`PickDismiss::Stay`] every pick is
/// heard while the menu is open.
pub(crate) fn picker<T: 'static>(
    dismiss: PickDismiss,
    closing: Signal<Closing>,
    onpick: EventHandler<T>,
    onclose: EventHandler<()>,
) -> EventHandler<T> {
    let mut closing = closing;
    EventHandler::new(move |value: T| {
        if *closing.peek() != Closing::No {
            return;
        }
        onpick.call(value);
        if dismiss == PickDismiss::Close {
            closing.set(Closing::Picked);
            onclose.call(());
        }
    })
}

/// `pick`, then the focus back to the panel when the menu stays open, floats and holds the
/// keyboard: Blitz ends a click on a row by clearing the focus, and the next Up, Down or Enter
/// of a checklist must still reach the menu. A menu that closes, an inline one and one a field
/// drives leave the focus alone.
pub(crate) fn kept_focus<T: 'static>(
    pick: EventHandler<T>,
    tracker: Tracker,
    (dismiss, flow, active): (PickDismiss, Flow, Cursor),
) -> EventHandler<T> {
    match (dismiss, flow, active) {
        (PickDismiss::Stay, Flow::Floating, Cursor::Auto) => EventHandler::new(move |value: T| {
            pick.call(value);
            tracker.refocus();
        }),
        (PickDismiss::Close, _, _)
        | (PickDismiss::Stay, Flow::Inline, _)
        | (PickDismiss::Stay, Flow::Floating, Cursor::Controlled(_)) => pick,
    }
}
