//! A [`Machine`] as a hook: [`use_machine`] owns the state, stamps every step with a
//! [`FrameClock`] on `ds_core::time`'s clock (the wall clock, or a harness's virtual one), sleeps
//! until the machine's [`Machine::wake`], and hands each output to the surface's effect handler.
//! Nothing here decides: the surface's `io.rs` feeds inputs and carries out outputs. The trait,
//! `Elapsed` and `Stamp` are `ds_core`'s (`ds::base::machine`, `ds::base::time::stamp`).

use ds_core::machine::{Elapsed, Machine};
use ds_core::time::clock::sleep;
use ds_core::time::stamp::{FrameClock, Stamp};
use std::collections::VecDeque;
use std::time::Duration;

use dioxus::core::Task;
use dioxus::prelude::*;

/// A running [`Machine`]: send it inputs, read its state. Copy: signals and cells only.
pub struct MachineRef<M: Machine> {
    state: Signal<M>,
    params: CopyValue<M::Params>,
    on_out: CopyValue<Box<dyn FnMut(M::Out)>>,
    pending: CopyValue<VecDeque<M::Out>>,
    draining: CopyValue<Drain>,
    waking: CopyValue<Option<Task>>,
    clock: FrameClock,
}

/// Whether outputs are being handed out, so an input sent from a handler queues its outputs
/// behind the ones being handled instead of running the handler again inside itself.
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

/// Run a machine `M` on `params`, giving each output to `on_out`. The machine starts at its
/// default; it steps on [`MachineRef::send`] and whenever it asked to wake.
pub fn use_machine<M: Machine>(
    params: M::Params,
    on_out: impl FnMut(M::Out) + 'static,
) -> MachineRef<M> {
    let state = use_signal(M::default);
    let mut wanted = use_hook(|| CopyValue::new(params.clone()));
    if *wanted.peek() != params {
        wanted.set(params);
    }
    let mut handler =
        use_hook(|| CopyValue::new(Box::new(|_: M::Out| {}) as Box<dyn FnMut(M::Out)>));
    handler.set(Box::new(on_out));
    MachineRef {
        state,
        params: wanted,
        on_out: handler,
        pending: use_hook(|| CopyValue::new(VecDeque::new())),
        draining: use_hook(|| CopyValue::new(Drain::Idle)),
        waking: use_hook(|| CopyValue::new(None)),
        clock: use_hook(FrameClock::started),
    }
}

impl<M: Machine> MachineRef<M> {
    /// Step the machine with `input` now, and carry out what it says.
    pub fn send(&self, input: M::In) {
        self.advance(input);
    }

    /// The machine's state, live.
    pub fn state(&self) -> ReadSignal<M> {
        self.state.into()
    }

    /// Replace the parameters the next step reads, now. [`use_machine`] takes them at each render,
    /// which is one render behind a machine whose parameters are derived from its own state (a
    /// zoom step reads the scale the last step produced); a surface in that position computes
    /// them from the state it just read and sets them here before it sends.
    pub fn set_params(&self, params: M::Params) {
        let mut wanted = self.params;
        if *wanted.peek() != params {
            wanted.set(params);
        }
    }

    fn advance(mut self, input: M::In) {
        let now = self.clock.now();
        let (next, outs) = self
            .state
            .peek()
            .clone()
            .step(input, now, &self.params.peek());
        let mut state = self.state;
        if *state.peek() != next {
            state.set(next);
        }
        self.pending.write().extend(outs);
        self.drain();
        self.rewake(now);
    }

    fn drain(mut self) {
        let mut draining = self.draining;
        if *draining.peek() == Drain::Running {
            return;
        }
        draining.set(Drain::Running);
        loop {
            let next = self.pending.write().pop_front();
            let Some(out) = next else { break };
            (self.on_out.write())(out);
        }
        draining.set(Drain::Idle);
    }

    /// Sleep until the machine's next wake, replacing any earlier sleep.
    fn rewake(self, now: Stamp) {
        let mut waking = self.waking;
        if let Some(task) = waking.take() {
            task.cancel();
        }
        if let Some(at) = self.state.peek().wake() {
            let wait = Duration::from_millis(at.since(now));
            waking.set(Some(spawn(async move {
                sleep(wait).await;
                self.advance(M::In::from(Elapsed));
            })));
        }
    }
}
