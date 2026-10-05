//! The pointer hooks a row hands its caller: the sender's name and the time each
//! open a hover card of their own (design/06-INTERACTIONS.md section 3: `sender` and `time`
//! targets inside a `thread` row), and the row itself starts a drag on a press and opens the
//! thread card on entry. quire draws the parts, so it attaches the caller's handlers to them.

use crate::components::lists::row::row::relay;
use dioxus::prelude::*;
use ds_core::machine::{Elapsed, Machine};
use ds_core::time::stamp::Stamp;
use ds_motion::machine::{MachineRef, use_machine};
use ds_style::tokens::delay::DelayToken;
use std::time::Duration;

/// A part's pointer entering and leaving, as the events themselves (a caller reads the point to
/// place its card, or the element to anchor it).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartHooks {
    /// The pointer came over the part. It does not reach the row: a pointer entering a part is
    /// already inside the row, so the row's own `onpointerenter` has fired or will not.
    pub onpointerenter: EventHandler<PointerEvent>,
    /// The pointer left the part, for the row or anywhere else: a card over the rows, another
    /// row, outside the window. Which one is not known yet when this runs; a caller that opens
    /// the row's own card when the pointer is back on the row does that from `ThreadRow`'s
    /// `onpointerback`, not from here (FINDINGS "Pointer events").
    pub onpointerleave: EventHandler<PointerEvent>,
}

/// Where the pointer is, as the row's own enter and leave last said.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OnRow {
    On,
    Off,
}

/// The row's pointer: where it is, and until when a part's leave waits to see whether the pointer
/// came back to the row. A [`Machine`], so any crossing replaces the wait and nothing is left to
/// cancel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RowPointer {
    on_row: OnRow,
    wait: Option<Stamp>,
}

/// What moves the row's pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RowIn {
    /// The pointer crossed onto or off the row, or entered a part (which is inside the row).
    Crossed(OnRow),
    /// The pointer left a part: wait the close grace, then say whether it is on the row.
    PartLeft,
    /// The grace's end came.
    Elapsed,
}

impl From<Elapsed> for RowIn {
    fn from(_: Elapsed) -> Self {
        RowIn::Elapsed
    }
}

impl Machine for RowPointer {
    type In = RowIn;
    /// The pointer is back on the row.
    type Out = ();
    /// The close grace: `DelayToken::CardClose`.
    type Params = Duration;
    type Ctx = ();

    fn step(self, input: RowIn, at: Stamp, grace: &Duration, _: &()) -> (RowPointer, Vec<()>) {
        match input {
            RowIn::Crossed(on_row) => (RowPointer { on_row, wait: None }, Vec::new()),
            RowIn::PartLeft => (
                RowPointer {
                    wait: Some(at.after_span(*grace)),
                    ..self
                },
                Vec::new(),
            ),
            RowIn::Elapsed => match (self.wait, self.on_row) {
                (Some(until), OnRow::On) if at >= until => {
                    (RowPointer { wait: None, ..self }, vec![()])
                }
                (Some(until), OnRow::Off) if at >= until => {
                    (RowPointer { wait: None, ..self }, Vec::new())
                }
                (Some(_) | None, OnRow::On | OnRow::Off) => (self, Vec::new()),
            },
        }
    }

    fn wake(&self) -> Option<Stamp> {
        self.wait
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
    pointer: MachineRef<RowPointer>,
    /// The event of the part's leave, handed to the caller when the pointer is back.
    left: CopyValue<Option<PointerEvent>>,
    handler: Option<EventHandler<PointerEvent>>,
}

/// The row's [`Back`], for `handler`.
pub(crate) fn use_back(handler: Option<EventHandler<PointerEvent>>) -> Back {
    let left = use_hook(|| CopyValue::new(None::<PointerEvent>));
    let pointer = use_machine(
        |_| RowPointer {
            on_row: OnRow::Off,
            wait: None,
        },
        DelayToken::CardClose.delay(),
        || (),
        move |(), _| {
            let event = left.try_peek().ok().and_then(|event| event.clone());
            if let (Some(handler), Some(event)) = (handler, event) {
                handler.call(event);
            }
        },
    );
    Back {
        pointer,
        left,
        handler,
    }
}

impl Back {
    fn cross(self, on_row: OnRow) {
        self.pointer.send(RowIn::Crossed(on_row));
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
    /// grace, if the pointer has crossed nothing since. Where the row's enter and leave put the
    /// pointer stands: Blitz sends the row's leave before the part's, and it must stay.
    pub(crate) fn part_leave(self, hooks: Option<PartHooks>) -> impl FnMut(PointerEvent) + 'static {
        move |event: PointerEvent| {
            leave(hooks)(event.clone());
            if self.handler.is_none() {
                return;
            }
            let mut left = self.left;
            left.set(Some(event));
            self.pointer.send(RowIn::PartLeft);
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

#[cfg(test)]
mod tests {
    use super::{OnRow, RowIn, RowPointer};
    use ds_core::machine::Machine;
    use ds_core::time::stamp::Stamp;
    use std::time::Duration;

    fn pointer(on_row: OnRow, wait: Option<u64>) -> RowPointer {
        RowPointer {
            on_row,
            wait: wait.map(Stamp),
        }
    }

    /// Name, state before, input, time, state after, whether it says back, next wake.
    type Case = (
        &'static str,
        RowPointer,
        RowIn,
        u64,
        RowPointer,
        bool,
        Option<u64>,
    );

    #[test]
    fn a_part_leave_says_back_only_if_nothing_was_crossed_during_the_grace() {
        use OnRow::{Off, On};
        let grace = Duration::from_millis(150);
        #[rustfmt::skip]
        let cases: &[Case] = &[
            ("a part's leave starts the wait", pointer(On, None), RowIn::PartLeft, 100, pointer(On, Some(250)), false, Some(250)),
            ("the grace ends on the row: back", pointer(On, Some(250)), RowIn::Elapsed, 250, pointer(On, None), true, None),
            ("woken early: nothing", pointer(On, Some(250)), RowIn::Elapsed, 249, pointer(On, Some(250)), false, Some(250)),
            ("the grace ends off the row: nothing", pointer(Off, Some(250)), RowIn::Elapsed, 250, pointer(Off, None), false, None),
            ("the row's leave cancels the wait", pointer(On, Some(250)), RowIn::Crossed(Off), 200, pointer(Off, None), false, None),
            ("a part entered again cancels it too", pointer(On, Some(250)), RowIn::Crossed(On), 200, pointer(On, None), false, None),
            ("the row left before the part: the pointer stays off", pointer(Off, None), RowIn::PartLeft, 100, pointer(Off, Some(250)), false, Some(250)),
            ("a wake with no wait: nothing", pointer(On, None), RowIn::Elapsed, 900, pointer(On, None), false, None),
        ];
        for (name, from, input, at, state, back, wake) in cases {
            let (next, out) = from.step(*input, Stamp(*at), &grace, &());
            assert_eq!(next, *state, "{name}: state");
            assert_eq!(!out.is_empty(), *back, "{name}: back");
            assert_eq!(next.wake(), wake.map(Stamp), "{name}: wake");
        }
    }
}
