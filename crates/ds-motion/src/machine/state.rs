//! The state a machine hook runs over: the signal that holds it and the clock its stamps count on.

use dioxus::prelude::*;
use ds_core::machine::Machine;
use ds_core::time::stamp::{FrameClock, Stamp};

/// A machine's state and the clock its [`Stamp`]s count on. Copy: a signal and a clock. The
/// caller of [`use_machine_in`](super::use_machine_in) makes one with [`use_machine_state`], and
/// may put it in a context, read it from a sibling, persist it, or write it: the hook follows
/// the state's `wake()` whatever wrote it.
pub struct MachineState<M: Machine> {
    /// The state.
    pub state: Signal<M>,
    /// The clock every deadline in the state counts on; a seeded state is stamped with it.
    pub clock: FrameClock,
}

impl<M: Machine> Clone for MachineState<M> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<M: Machine> Copy for MachineState<M> {}

impl<M: Machine> std::fmt::Debug for MachineState<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MachineState")
    }
}

/// A machine state seeded by `initial`, which is given the stamp of this moment. The clock is the
/// root's [`FrameClock`] when a daemon provides one as context (so every machine on the surface
/// counts from the daemon's origin), else one that starts now.
pub fn use_machine_state<M: Machine>(initial: impl FnOnce(Stamp) -> M) -> MachineState<M> {
    let clock =
        use_hook(|| try_consume_context::<FrameClock>().unwrap_or_else(FrameClock::started));
    let state = use_signal(|| initial(clock.now()));
    MachineState { state, clock }
}
