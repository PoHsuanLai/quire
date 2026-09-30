//! One [`HoverIntent`] machine and its timers for a component whose single part opens when the
//! pointer rests on it (the edge peek, the link pill): the events go in, `shown` comes out. The
//! timers are tasks of the component's scope and every write they make is a `try_set`, so one
//! that outlives its component stops instead of panicking.

use dioxus::core::{ScopeId, current_scope_id};
use dioxus::prelude::*;
use ds_core::time::clock::{now, sleep};
use ds_core::vocab::Shown;
use ds_motion::hover_intent::{HoverEvent, HoverIntent, HoverProfile, IntentEffect, IntentPhase};
use ds_style::task::{Gone, spawn_in, try_get, try_set_if_changed};
use std::time::Duration;

/// There is one thing to hover, so the machine's key is nothing.
type Part = ();

/// The machine, the profile it waits by and the scope its timers run in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoverOpen {
    intent: Signal<HoverIntent<Part>>,
    profile: HoverProfile,
    scope: ScopeId,
}

/// A [`HoverOpen`] for this component, waiting by `profile`.
pub fn use_hover_open(profile: HoverProfile) -> HoverOpen {
    HoverOpen {
        intent: use_signal(HoverIntent::default),
        profile,
        scope: use_hook(current_scope_id),
    }
}

impl HoverOpen {
    /// The pointer came over the part: open after the profile's wait (at once while warm).
    pub fn over(self) {
        self.feed(HoverEvent::Over((), self.profile));
    }

    /// The pointer left the part: close after the profile's close delay.
    pub fn out(self) {
        self.feed(HoverEvent::Out);
    }

    /// The pointer came over what the part opened (the peeking sidebar): keep it open.
    pub fn enter_card(self) {
        self.feed(HoverEvent::EnterCard);
    }

    /// The pointer left what the part opened.
    pub fn leave_card(self) {
        self.feed(HoverEvent::LeaveCard);
    }

    /// Whether it is open: open, or closing (which still shows).
    pub fn shown(self) -> Shown {
        match self.intent.read().phase() {
            IntentPhase::Open { .. } | IntentPhase::Closing { .. } => Shown::Visible,
            IntentPhase::Idle | IntentPhase::Pending { .. } => Shown::Hidden,
        }
    }

    /// Back to nothing pending and nothing open, at once (a click acted).
    pub fn reset(self) {
        self.feed(HoverEvent::ClickInList);
    }

    fn feed(self, event: HoverEvent<Part>) {
        let _ = self.try_feed(event);
    }

    fn try_feed(self, event: HoverEvent<Part>) -> Result<(), Gone> {
        let (next, effect) = try_get(self.intent)?.step(event, now());
        try_set_if_changed(self.intent, next)?;
        match effect {
            IntentEffect::StartOpen { after } => self.after(after, HoverEvent::OpenDue),
            IntentEffect::StartClose { after } => self.after(after, HoverEvent::CloseDue),
            IntentEffect::None
            | IntentEffect::CancelOpen
            | IntentEffect::CancelClose
            | IntentEffect::Open(_)
            | IntentEffect::Close(_)
            | IntentEffect::Remove(_)
            | IntentEffect::Peek(_) => {}
        }
        Ok(())
    }

    /// Feed `event` once `wait` has passed. A stale timer is harmless: the machine reads the time.
    fn after(self, wait: Duration, event: HoverEvent<Part>) {
        let _ = spawn_in(self.scope, async move {
            sleep(wait).await;
            self.feed(event);
        });
    }
}
