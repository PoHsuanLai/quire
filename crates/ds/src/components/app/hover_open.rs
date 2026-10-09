//! One [`HoverIntent`] machine for a component whose single part opens when the pointer rests on
//! it (the edge peek, the link pill): the events go in, `shown` comes out. The machine's own
//! deadlines are its one timer (`ds::machine::use_machine`), so a timer that outlives its
//! component is dropped with the component's scope.

use dioxus::prelude::*;
use ds_core::vocab::Shown;
use ds_motion::hover_intent::{HoverEvent, HoverIntent, HoverProfile, IntentPhase};
use ds_motion::machine::{MachineRef, use_machine};
use ds_style::tokens::delay::TipDelay;

/// There is one thing to hover, so the machine's key is nothing.
type Part = ();

/// The machine and the profile it waits by.
#[derive(Debug, Clone, Copy)]
pub struct HoverOpen {
    machine: MachineRef<HoverIntent<Part>>,
    profile: HoverProfile,
}

/// A [`HoverOpen`] for this component, waiting by `profile`.
pub fn use_hover_open(profile: HoverProfile) -> HoverOpen {
    HoverOpen {
        machine: use_machine(
            |_| HoverIntent::default(),
            TipDelay::default(),
            || (),
            |_, _| {},
        ),
        profile,
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
        match self.machine.state().read().phase() {
            IntentPhase::Open { .. } | IntentPhase::Closing { .. } => Shown::Visible,
            IntentPhase::Idle | IntentPhase::Pending { .. } => Shown::Hidden,
        }
    }

    /// Back to nothing pending and nothing open, at once (a click acted).
    pub fn reset(self) {
        self.feed(HoverEvent::ClickInList);
    }

    fn feed(self, event: HoverEvent<Part>) {
        self.machine.send(event);
    }
}
