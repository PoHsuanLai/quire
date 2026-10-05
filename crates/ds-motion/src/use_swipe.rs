//! The swipe machine as a hook: [`SwipeState`] run by [`use_machine`], fed from a card's pointer
//! and wheel events. The quiet spell after a scroll is the machine's own deadline, so the hook has
//! no timer of its own.

use super::machine::{MachineRef, use_machine};
use super::swipe::{Click, SwipeEffect, SwipeInput, SwipeMetrics, SwipeState};
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::time::stamp::Stamp;
use ds_core::vocab::PressPhase;

/// A live swipe: read its state in render, feed it from the card's listeners.
#[derive(Debug, Clone, Copy)]
pub struct Swiper {
    machine: MachineRef<SwipeState>,
}

impl Swiper {
    /// Where the swipe is.
    pub fn state(&self) -> SwipeState {
        *self.machine.state().read()
    }

    /// Feed one input, stamped now by the machine; a dismissal is reported to the hook's
    /// `on_dismiss`.
    pub fn feed(&self, input: SwipeInput) {
        self.machine.send(input);
    }

    /// The input a pointer move makes: a move while the primary button is down, or the release
    /// Blitz never delivered (the button came up outside the card; it has no pointer capture).
    pub fn pointer_moved(&self, x: Px, held: PressPhase) {
        match held {
            PressPhase::Pressed | PressPhase::Held => self.feed(SwipeInput::Move { x }),
            PressPhase::Idle => self.feed(SwipeInput::Up),
        }
    }

    /// A click was heard: whether it is a press on the card (not the end of a drag).
    pub fn take_click(&self) -> Click {
        let click = self.machine.state().peek().click();
        self.feed(SwipeInput::Clicked);
        click
    }

    /// Now, as the machine's stamp.
    pub fn now(&self) -> Stamp {
        self.machine.now()
    }
}

/// A swipe with `metrics`, calling `on_dismiss` the moment a gesture ends past a threshold.
pub fn use_swipe(metrics: SwipeMetrics, on_dismiss: EventHandler<()>) -> Swiper {
    Swiper {
        machine: use_machine(
            |_| SwipeState::default(),
            metrics,
            || (),
            move |SwipeEffect::Dismiss, _| on_dismiss.call(()),
        ),
    }
}
