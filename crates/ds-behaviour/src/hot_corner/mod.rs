//! A hot corner's trigger machine (design/13 §13.3.12; 03 §1.10): the pointer resting in a
//! corner for the dwell fires it once; it fires again only after the pointer has left and the
//! re-arm has run out, in either order.
//!
//! A [`Corner`] is a `Machine`: the dwell and the re-arm are deadlines in its state, so its wake
//! is the earlier of the two that run and no timer identity is needed. The caller feeds the
//! pointer's enter and leave and runs the one clock that wakes it. Modifier gating (a corner that
//! acts only while a modifier is held) stays with the caller, which feeds `Enter` only while the
//! corner's modifiers are down.

mod model;
mod step;
#[cfg(test)]
mod tests;

pub use model::{Corner, CornerIn, CornerOut, CornerParams, Inside, Rearm};
