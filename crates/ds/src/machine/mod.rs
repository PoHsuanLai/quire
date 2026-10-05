//! A [`Machine`](ds_core::machine::Machine) as a hook: [`use_machine`] keeps the state, seeded by the
//! caller, and [`use_machine_in`] runs a machine over a [`MachineState`] the caller owns. The
//! hooks live in `ds_motion::machine`, below the hooks that run on them (presence, rosters, the
//! swipe); this is their name in `ds` for surfaces and apps. The trait, `Elapsed` and `Stamp` are
//! `ds_core`'s (`ds::base::machine`, `ds::base::time::stamp`).

pub use ds_motion::machine::{
    MachineRef, MachineState, use_machine, use_machine_in, use_machine_state,
};
