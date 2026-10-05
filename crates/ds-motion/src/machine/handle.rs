//! The running machine: send it inputs, read its state. Copy: signals and cells only.

use super::state::MachineState;
use super::waking::Waking;
use dioxus::core::{ScopeId, queue_effect};
use dioxus::prelude::*;
use ds_core::machine::{Elapsed, Machine};
use ds_core::time::stamp::Stamp;
use ds_style::task::try_set_if_changed;
use std::collections::VecDeque;

/// A running [`Machine`]: send it inputs, read its state. Copy: signals and cells only.
pub struct MachineRef<M: Machine> {
    pub(super) held: MachineState<M>,
    pub(super) params: CopyValue<M::Params>,
    pub(super) ctx: CopyValue<Option<CtxReader<M>>>,
    pub(super) on_out: CopyValue<OutHandler<M>>,
    pub(super) pending: CopyValue<Pending<M>>,
    pub(super) waking: CopyValue<Waking>,
    pub(super) scope: ScopeId,
}

/// What reads the caller's context at a step.
pub(super) type CtxReader<M> = Box<dyn Fn() -> <M as Machine>::Ctx>;

/// What carries out an output; it can send the machine a follow-up.
pub(super) type OutHandler<M> = Box<dyn FnMut(<M as Machine>::Out, MachineRef<M>)>;

/// The outputs waiting to be handed out, and whether that is going on, so an input sent from a
/// handler queues its outputs behind the ones being handled instead of running the handler again
/// inside itself.
pub(super) struct Pending<M: Machine> {
    outs: VecDeque<M::Out>,
    drain: Drain,
}

impl<M: Machine> Default for Pending<M> {
    fn default() -> Self {
        Pending {
            outs: VecDeque::new(),
            drain: Drain::Idle,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Drain {
    Idle,
    Running,
}

impl<M: Machine> Clone for MachineRef<M> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<M: Machine> Copy for MachineRef<M> {}

impl<M: Machine> std::fmt::Debug for MachineRef<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MachineRef")
    }
}

impl<M: Machine> MachineRef<M> {
    /// Step the machine with `input` now, and carry out what it says. From an event handler, a
    /// task or an effect; a component body uses [`Self::send_from_render`].
    pub fn send(&self, input: M::In) {
        if let Some(outs) = self.step_now(input) {
            self.finish(outs);
        }
    }

    /// Step the machine with `input` from a component body: the state changes at once, so this
    /// render draws it, and the outputs wait until after the render (a handler must not write
    /// signals during one). The timer follows the state through the hook's effect. Send before
    /// reading [`Self::state`] in the same render, so the render does not read what it then
    /// writes.
    pub fn send_from_render(&self, input: M::In) {
        let Some(outs) = self.step_now(input) else {
            return;
        };
        if outs.is_empty() {
            return;
        }
        let machine = *self;
        queue_effect(move || machine.finish(outs));
    }

    /// The machine's state, live.
    pub fn state(&self) -> ReadSignal<M> {
        self.held.state.into()
    }

    /// Now, as the machine's stamp: what an input stamped by the caller (a seeded deadline) counts
    /// on.
    pub fn now(&self) -> Stamp {
        self.held.clock.now()
    }

    /// Replace the parameters the next step reads, now. The hook takes them at each render, which
    /// is one render behind a machine whose parameters are derived from its own state (a zoom step
    /// reads the scale the last step produced); a surface in that position computes them from the
    /// state it just read and sets them here before it sends.
    pub fn set_params(&self, params: M::Params) {
        let mut wanted = self.params;
        let Ok(held) = wanted.try_peek().map(|held| held.clone()) else {
            return;
        };
        if held != params {
            wanted.set(params);
        }
    }

    /// Step with `input` and store the state; the outputs, or `None` when the machine's owner
    /// is gone.
    fn step_now(&self, input: M::In) -> Option<Vec<M::Out>> {
        let state = self.held.state.try_peek().ok()?.clone();
        let params = self.params.try_peek().ok()?.clone();
        let cx = (self.ctx.try_peek().ok()?.as_ref()?)();
        let (next, outs) = state.step(input, self.now(), &params, &cx);
        try_set_if_changed(self.held.state, next).ok()?;
        Some(outs)
    }

    /// Hand out `outs` and follow the new state's wake.
    pub(super) fn finish(self, outs: Vec<M::Out>) {
        let mut pending = self.pending;
        if let Ok(mut queue) = pending.try_write() {
            queue.outs.extend(outs);
        }
        self.drain();
        self.rewake();
    }

    fn drain(self) {
        let mut pending = self.pending;
        match pending.try_write() {
            Ok(mut queue) if queue.drain == Drain::Idle => queue.drain = Drain::Running,
            Ok(_) | Err(_) => return,
        }
        loop {
            let next = pending
                .try_write()
                .ok()
                .and_then(|mut queue| queue.outs.pop_front());
            let Some(out) = next else { break };
            let mut handler = self.on_out;
            if let Ok(mut handler) = handler.try_write() {
                handler(out, self);
            }
        }
        if let Ok(mut queue) = pending.try_write() {
            queue.drain = Drain::Idle;
        }
    }

    /// The clock alone woke the machine.
    pub(super) fn elapsed(self) {
        self.send(M::In::from(Elapsed));
    }
}
