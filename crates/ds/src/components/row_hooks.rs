//! The pointer hooks a row hands its caller (mailo gaps 2): the sender's name and the time each
//! open a hover card of their own (design/06-INTERACTIONS.md section 3: `sender` and `time`
//! targets inside a `thread` row), and the row itself starts a drag on a press and opens the
//! thread card on entry. quire draws the parts, so it attaches the caller's handlers to them.

use crate::appearance::MotionLevel;
use crate::time::sleep;
use crate::tokens::DelayToken;
use dioxus::prelude::*;

/// A part's pointer entering and leaving, as the events themselves (a caller reads the point to
/// place its card, or the element to anchor it).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartHooks {
    /// The pointer came over the part. It does not reach the row: a pointer entering a part is
    /// already inside the row, so the row's own `onpointerenter` has fired or will not.
    pub onpointerenter: EventHandler<PointerEvent>,
    /// The pointer left the part, for the row or anywhere else: a card over the rows, another
    /// row, outside the window. Which one is not known yet when this runs; a caller that opens
    /// the row's own card when the pointer is back on the row does that from `ListRow`'s
    /// `onpointerback`, not from here (FINDINGS "A part's leave is not the row's enter").
    pub onpointerleave: EventHandler<PointerEvent>,
}

/// Where the pointer is, as the row's own enter and leave last said.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OnRow {
    On,
    Off,
}

/// The row's pointer state, and a count of its crossings that a waiting `onpointerback` checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Crossings {
    on_row: OnRow,
    count: u64,
}

impl Crossings {
    /// One more crossing, ending `on_row`.
    fn cross(self, on_row: OnRow) -> Self {
        Crossings {
            on_row,
            count: self.count.wrapping_add(1),
        }
    }
}

/// The row's "back on the row" hook: the pointer left a part and rested on the row.
///
/// A part's leave says only that the pointer left the part: for the row, or for a card
/// floating over the rows, which a sender card's 6 px gap below its name makes a short trip
/// across the row. So the hook waits the hover card's close grace (`HoverClose`, the time the
/// hub gives a pointer to reach an open card) and fires only if nothing was crossed meanwhile:
/// no row leave (the pointer went to the card, another row or away), no part entered again.
/// The wait also makes it independent of the order a row's and a part's leave arrive in, which
/// Blitz and the UI Events order disagree on.
#[derive(Clone, Copy)]
pub(crate) struct Back {
    crossings: CopyValue<Crossings>,
    handler: Option<EventHandler<PointerEvent>>,
}

/// The row's [`Back`], for `handler`.
pub(crate) fn use_back(handler: Option<EventHandler<PointerEvent>>) -> Back {
    let crossings = use_hook(|| {
        CopyValue::new(Crossings {
            on_row: OnRow::Off,
            count: 0,
        })
    });
    Back { crossings, handler }
}

impl Back {
    fn cross(self, on_row: OnRow) {
        let mut crossings = self.crossings;
        let next = crossings.peek().cross(on_row);
        crossings.set(next);
    }

    /// The row's enter: note it, then call the caller's `handler`.
    pub(crate) fn enter(
        self,
        handler: Option<EventHandler<PointerEvent>>,
    ) -> impl FnMut(PointerEvent) {
        move |event| {
            self.cross(OnRow::On);
            relay(handler)(event);
        }
    }

    /// The row's leave: note it, then call the caller's `handler`.
    pub(crate) fn leave(
        self,
        handler: Option<EventHandler<PointerEvent>>,
    ) -> impl FnMut(PointerEvent) {
        move |event| {
            self.cross(OnRow::Off);
            relay(handler)(event);
        }
    }

    /// A part's enter: note it (a waiting `onpointerback` is off), then the caller's hook.
    pub(crate) fn part_enter(self, hooks: Option<PartHooks>) -> impl FnMut(PointerEvent) + 'static {
        move |event| {
            self.cross(OnRow::On);
            enter(hooks)(event);
        }
    }

    /// A part's leave: the caller's part hook at once, then `onpointerback` after the close
    /// grace, if the pointer has crossed nothing since.
    pub(crate) fn part_leave(self, hooks: Option<PartHooks>) -> impl FnMut(PointerEvent) + 'static {
        move |event: PointerEvent| {
            leave(hooks)(event.clone());
            let Some(back) = self.handler else {
                return;
            };
            // A crossing of its own, which keeps where the row's enter and leave put the pointer:
            // Blitz sends the row's leave before the part's, and it must stand.
            let on_row = self.crossings.peek().on_row;
            self.cross(on_row);
            let crossings = self.crossings;
            let left = *crossings.peek();
            spawn(async move {
                sleep(DelayToken::HoverClose.delay(MotionLevel::Standard)).await;
                let now = *crossings.peek();
                if now == left && now.on_row == OnRow::On {
                    back.call(event);
                }
            });
        }
    }
}

/// The enter listener of `hooks`: a part with no hooks carries no handler.
fn enter(hooks: Option<PartHooks>) -> impl FnMut(PointerEvent) + 'static {
    move |event| {
        if let Some(hooks) = hooks {
            hooks.onpointerenter.call(event);
        }
    }
}

/// The leave listener of `hooks`.
fn leave(hooks: Option<PartHooks>) -> impl FnMut(PointerEvent) + 'static {
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
