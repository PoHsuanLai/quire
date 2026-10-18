//! A deadline for every call quire makes to another process.
//!
//! A peer can be slow, crashed, wedged behind a password prompt or simply absent, and a bus
//! call has no timeout of its own. Wrapping the call in [`within`] gives it one: the call either
//! answers inside its [`Deadline`] or ends as [`NotAnswering`], which a screen shows as "X is not
//! answering" instead of spinning forever.
//!
//! The deadlines are named, one per kind of wait ([`Deadline::READ`], [`Deadline::START`],
//! [`Deadline::ACTION`], [`Deadline::PROMPT`]); a caller picks one and never writes a bare
//! duration. The crate names no runtime and no bus: it is a wall-clock delay and a future.

mod answer;
mod deadline;
mod within;

pub use answer::{SenderName, Verdict};
pub use deadline::Deadline;
pub use within::{NotAnswering, within};
