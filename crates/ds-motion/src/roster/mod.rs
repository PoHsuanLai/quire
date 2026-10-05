//! The list roster as a pure state machine: which keys are on screen and in what state, so a
//! leaving row stays in the tree until its exit settles and the rows below close the gap
//! (design/30 section 1.3: insert fades and slides down over `--t-move`, removal fades and
//! slides up over `--t-quick`, the rows below close the gap over `--t-move`).
//!
//! [`RosterState`] is a `Machine`: each leaving row keeps the time its exit settles, and the
//! entering and healing rows share one rest deadline, so its wake is the earlier of them and a
//! caller can run it with `use_machine` (the hook `use_roster` does) or by itself.

mod model;
mod ops;
mod step;
#[cfg(test)]
mod tests;

pub use model::{
    Heal, LeaveBy, Measured, RosterEntry, RosterIn, RosterOut, RosterParams, RosterState, RowPitch,
    StayError, Stayed, presence_slug,
};
