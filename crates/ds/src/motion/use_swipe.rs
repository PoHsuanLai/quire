//! The swipe machine as a hook: [`SwipeState`] in a signal, fed from a card's
//! pointer and wheel events, with the clock and the quiet timer a scroll needs. The machine is
//! pure (`swipe.rs`); this owns the time.

use super::swipe::{Click, Stamp, SwipeEffect, SwipeInput, SwipeMetrics, SwipeState};
use crate::core::geometry::units::Px;
use crate::style::task::{Gone, spawn_in, try_get, try_set};
use crate::core::time::clock::sleep;
use crate::core::vocab::PressPhase;
use crate::style::scope::{Scope, use_scope_signal};
use crate::style::tokens::delay::DelayToken;
use dioxus::core::{Task, current_scope_id};
use dioxus::prelude::*;
use std::time::Instant;

/// A live swipe: read its state in render, feed it from the card's listeners.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Swiper {
    state: Signal<SwipeState>,
    quiet: Signal<Option<Task>>,
    env: Signal<Scope>,
    scope: ScopeId,
    origin: Instant,
    metrics: SwipeMetrics,
    on_dismiss: EventHandler<()>,
}

impl Swiper {
    /// Where the swipe is.
    pub fn state(&self) -> SwipeState {
        *self.state.read()
    }

    /// Feed one input stamped now; a dismissal is reported to the hook's `on_dismiss`.
    pub fn feed(&self, input: SwipeInput) {
        let _ = self.try_feed(input);
    }

    /// The input a pointer move makes: a move while the primary button is down, or the release
    /// Blitz never delivered (the button came up outside the card; it has no pointer capture).
    pub fn pointer_moved(&self, x: Px, held: PressPhase) {
        match held {
            PressPhase::Pressed | PressPhase::Held => {
                self.feed(SwipeInput::Move { x, at: self.now() })
            }
            PressPhase::Idle => self.feed(SwipeInput::Up { at: self.now() }),
        }
    }

    /// A click was heard: whether it is a press on the card (not the end of a drag).
    pub fn take_click(&self) -> Click {
        let state = *self.state.peek();
        let _ = try_set(self.state, state.clicked());
        state.click()
    }

    /// Now, as the machine's stamp.
    pub fn now(&self) -> Stamp {
        Stamp(crate::core::time::clock::since(self.origin))
    }

    fn try_feed(&self, input: SwipeInput) -> Result<(), Gone> {
        let (next, effect) = try_get(self.state)?.step(input, self.metrics);
        if next != *self.state.peek() {
            try_set(self.state, next)?;
        }
        match effect {
            SwipeEffect::None => Ok(()),
            SwipeEffect::Dismiss => {
                self.on_dismiss.call(());
                Ok(())
            }
            SwipeEffect::ArmQuiet => self.arm_quiet(),
        }
    }

    /// (Re)start the quiet timer: when it runs out with no new delta, the scroll is decided.
    fn arm_quiet(&self) -> Result<(), Gone> {
        if let Some(pending) = try_get(self.quiet)? {
            pending.cancel();
        }
        try_get(self.env)?;
        let wait = DelayToken::SwipeQuiet.delay();
        let swiper = *self;
        let task = spawn_in(self.scope, async move {
            sleep(wait).await;
            if try_set(swiper.quiet, None).is_ok() {
                swiper.feed(SwipeInput::Quiet);
            }
        });
        try_set(self.quiet, Some(task))
    }
}

/// A swipe with `metrics`, calling `on_dismiss` the moment a gesture ends past a threshold.
pub fn use_swipe(metrics: SwipeMetrics, on_dismiss: EventHandler<()>) -> Swiper {
    Swiper {
        state: use_signal(SwipeState::default),
        quiet: use_signal(|| None),
        env: use_scope_signal(),
        scope: use_hook(current_scope_id),
        origin: use_hook(crate::core::time::clock::now),
        metrics,
        on_dismiss,
    }
}
