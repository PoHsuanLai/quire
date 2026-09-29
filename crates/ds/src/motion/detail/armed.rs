//! A press kept for the change it causes (design/26-DETAILS.md R5). A control's press and the
//! state change it asks for arrive in different renders: the press in a handler, the new state
//! when the service answers. An [`Armed`] keeps the press's [`Contact`] until a change spends it,
//! so that change (and only that one) may spring. A Pending change passes the press on to the
//! operation's end, so the success of a join the person clicked springs however long it took; a
//! press that changes nothing goes stale after `PendingCap`, and a later change from elsewhere is
//! `Touch::Remote`.

use super::cue::Cue;
use super::moment::Moment;
use super::operation::Deadline;
use super::touch::{Contact, Touch};
use dioxus::prelude::*;
use std::time::Instant;

/// How long a kept press lasts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hold {
    /// Pressed at this instant and nothing has changed since: stale after `PendingCap`.
    Since(Instant),
    /// The press started an operation that is still pending: kept until it ends.
    Operation,
}

/// A kept press and the last change it was offered to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Armed {
    press: CopyValue<Option<(Contact, Hold)>>,
    seen: CopyValue<Option<u32>>,
}

/// A place to keep a press for the change it causes.
pub(crate) fn use_armed() -> Armed {
    Armed {
        press: use_hook(|| CopyValue::new(None)),
        seen: use_hook(|| CopyValue::new(None)),
    }
}

impl Armed {
    /// Keep `touch` (from a handler) for the next change; a remote touch keeps nothing.
    pub(crate) fn arm(mut self, touch: Touch) {
        match touch {
            Touch::Contact(contact) => self.press.set(Some((
                contact,
                Hold::Since(crate::core::time::clock::now()),
            ))),
            Touch::Remote => {}
        }
    }

    /// Who is causing a change made now: the kept press while it holds.
    pub(crate) fn touch(self) -> Touch {
        match *self.press.peek() {
            Some((contact, Hold::Operation)) => Touch::Contact(contact),
            Some((contact, Hold::Since(at)))
                if crate::core::time::clock::since(at) <= Deadline::cap().length() =>
            {
                Touch::Contact(contact)
            }
            Some(_) | None => Touch::Remote,
        }
    }

    /// `cue` is the change being drawn: the first render of a new change spends the press, or,
    /// for a Pending change the press caused, hands it to the operation's end.
    pub(crate) fn spend(mut self, cue: Cue) {
        let serial = Some(cue.serial());
        if *self.seen.peek() == serial {
            return;
        }
        let first = self.seen.peek().is_none();
        self.seen.set(serial);
        if first {
            return;
        }
        let kept = match (*self.press.peek(), cue.moment(), cue.touch()) {
            (Some((contact, _)), Moment::Pending, Touch::Contact(_)) => {
                Some((contact, Hold::Operation))
            }
            _ => None,
        };
        self.press.set(kept);
    }
}
