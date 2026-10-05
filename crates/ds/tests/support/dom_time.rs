//! A bare `VirtualDom` on a virtual clock: the SSR tests that need a component's timers to run
//! (a toast's hold, a roster's settle) advance one clock by exactly the span they name instead
//! of waiting on the wall clock, so what they see is the same on an idle machine and a loaded one.

#![allow(dead_code)] // Each test binary that includes this file uses part of it.

use dioxus::core::NoOpMutations;
use dioxus::prelude::*;
use ds_core::time::clock::{ClockGuard, VirtualClock};
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::pin;
use std::task::{Context, Waker};
use std::time::Duration;

/// A dom whose `ds_core::time` clock is virtual, from before its first render until it drops.
pub struct TimedDom {
    /// First, so the dom (and every task sleeping on the clock) drops while it is installed.
    dom: VirtualDom,
    clock: VirtualClock,
    _installed: ClockGuard,
}

impl TimedDom {
    /// Install a virtual clock on this thread, then build `dom` on it: pass the dom's
    /// constructor, so nothing reads the clock before it is installed.
    pub fn new(make: impl FnOnce() -> VirtualDom) -> Self {
        let clock = VirtualClock::new();
        let installed = clock.install();
        let mut dom = make();
        dom.rebuild_in_place();
        TimedDom {
            dom,
            clock,
            _installed: installed,
        }
    }

    /// Let `span` of virtual time pass: stop at each timer due inside it, in order, rendering
    /// whatever the dom queued at that instant, then at the end of the span (a timer due exactly
    /// there fires too).
    pub fn run_for(&mut self, span: Duration) {
        let end = self.clock.elapsed().saturating_add(span);
        self.render_queued();
        while let Some(due) = self.clock.next_due().filter(|due| *due <= end) {
            self.clock.advance_to(due);
            self.render_queued();
        }
        self.clock.advance_to(end);
        self.render_queued();
    }

    /// Step to each timer in turn until `done` holds on the dom's markup, for at most `bound` of
    /// virtual time. Returns the instant it first held (time since the dom was built), or `None`
    /// when it never did.
    pub fn run_until(
        &mut self,
        bound: Duration,
        done: impl Fn(&VirtualDom) -> bool,
    ) -> Option<Duration> {
        let end = self.clock.elapsed().saturating_add(bound);
        loop {
            self.render_queued();
            if done(&self.dom) {
                return Some(self.clock.elapsed());
            }
            let due = self.clock.next_due().filter(|due| *due <= end)?;
            self.clock.advance_to(due);
        }
    }

    /// Poll the dom's tasks and render until nothing is left to do at this instant.
    fn render_queued(&mut self) {
        let mut cx = Context::from_waker(Waker::noop());
        loop {
            let ready = pin!(self.dom.wait_for_work()).poll(&mut cx).is_ready();
            if !ready {
                return;
            }
            self.dom.render_immediate(&mut NoOpMutations);
        }
    }
}

impl Deref for TimedDom {
    type Target = VirtualDom;

    fn deref(&self) -> &VirtualDom {
        &self.dom
    }
}

impl DerefMut for TimedDom {
    fn deref_mut(&mut self) -> &mut VirtualDom {
        &mut self.dom
    }
}
