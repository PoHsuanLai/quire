//! The Command key's own gestures, read on the Mac view of the keyboard (03 §2.14): a double
//! tap summons the companion ([`Tap`], ux §4.2) and a hold talks to it ([`HoldKey`], voice
//! §4.1). A Command press that meets another key or the pointer is a chord (Command-C,
//! Command-click, Command-drag) and neither gesture fires.
//!
//! [`HoldKey`] sees every Command edge and reports a lone short press as [`HoldOut::Tap`]; the
//! caller feeds each one to [`Tap`] as [`TapIn::Tap`], which decides the double.

mod hold;
mod tap;
#[cfg(test)]
mod tests;

pub use hold::{CancelCause, HoldIn, HoldKey, HoldOut, HoldParams, KeyEdge};
pub use tap::{Tap, TapIn, TapOut, TapParams};
