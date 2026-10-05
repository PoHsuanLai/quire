//! The two hook forms: the owned one, seeded by the caller, and the controlled one over a
//! [`MachineState`] the caller holds. There is one implementation: the owned form is the
//! controlled form over a state the hook made.

use super::handle::{CtxReader, MachineRef, OutHandler, Pending};
use super::state::{MachineState, use_machine_state};
use super::waking::Waking;
use dioxus::core::current_scope_id;
use dioxus::prelude::*;
use ds_core::machine::Machine;
use ds_core::time::stamp::Stamp;

/// Run a machine `M` that starts at `initial(stamp of now)`, on `params`, giving each output to
/// `on_out` with the machine so it can send a follow-up. `ctx` is read at every step, including
/// the step a wake causes. The machine steps on [`MachineRef::send`] and whenever it asked to
/// wake; a machine at rest runs no timer.
pub fn use_machine<M: Machine>(
    initial: impl FnOnce(Stamp) -> M,
    params: M::Params,
    ctx: impl Fn() -> M::Ctx + 'static,
    on_out: impl FnMut(M::Out, MachineRef<M>) + 'static,
) -> MachineRef<M> {
    use_machine_in(use_machine_state(initial), params, ctx, on_out)
}

/// Run a machine over `held`, which the caller made with [`use_machine_state`] and owns: it can
/// read or write the state elsewhere, and the timer follows the state's `wake()` whoever wrote it.
/// Otherwise as [`use_machine`].
pub fn use_machine_in<M: Machine>(
    held: MachineState<M>,
    params: M::Params,
    ctx: impl Fn() -> M::Ctx + 'static,
    on_out: impl FnMut(M::Out, MachineRef<M>) + 'static,
) -> MachineRef<M> {
    let mut wanted = use_hook(|| CopyValue::new(params.clone()));
    if *wanted.peek() != params {
        wanted.set(params);
    }
    let mut reader = use_hook(|| CopyValue::new(None::<CtxReader<M>>));
    reader.set(Some(Box::new(ctx)));
    let mut handler =
        use_hook(|| CopyValue::new(Box::new(|_: M::Out, _: MachineRef<M>| {}) as OutHandler<M>));
    handler.set(Box::new(on_out));
    let machine = MachineRef {
        held,
        params: wanted,
        ctx: reader,
        on_out: handler,
        pending: use_hook(|| CopyValue::new(Pending::default())),
        waking: use_hook(|| CopyValue::new(Waking::default())),
        scope: use_hook(current_scope_id),
    };
    // Follow the state: whatever wrote it, the timer asks the state when to wake.
    use_effect(move || {
        let _ = held.state.read();
        machine.rewake();
    });
    machine
}
