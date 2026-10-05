//! A [`Machine`](ds_core::machine::Machine) as a hook. [`use_machine`] keeps the state, seeded by
//! the caller; [`use_machine_in`] runs the machine over a [`MachineState`] the caller made and
//! owns. Both stamp every step with the state's [`FrameClock`](ds_core::time::stamp::FrameClock)
//! on `ds_core::time`'s clock (the wall clock, or a harness's virtual one), read the caller's
//! context at each step, sleep until [`Machine::wake`](ds_core::machine::Machine::wake), and hand
//! each output to the surface's handler together with the [`MachineRef`], so it can send a
//! follow-up. Nothing here decides: the surface feeds inputs and carries out outputs. The trait,
//! `Elapsed` and `Stamp` are `ds_core`'s (`ds::base::machine`, `ds::base::time::stamp`).

mod handle;
mod hook;
mod state;
mod waking;

pub use handle::MachineRef;
pub use hook::{use_machine, use_machine_in};
pub use state::{MachineState, use_machine_state};
