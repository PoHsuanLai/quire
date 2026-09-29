//! The grammar of small state details (design/26-DETAILS.md), enforced by types.
//!
//! A component keeps its state as its own enum and implements [`Detailed`] for it: the one place
//! that says what each transition means, as an exhaustive `match`. [`use_detail`] turns the state
//! into a [`Detail`], whose [`Cue`] is the only thing the primitives take, so a component cannot
//! play a moment its table does not name. A pending loop needs a [`PendingToken`]; a spring's
//! release velocity comes from a [`Contact`], which only a pointer or key handler's event can
//! give; Reduced motion is inside every primitive, and every primitive's clock stops when its
//! moment ends.

pub mod armed;
pub mod cue;
pub mod detailed;
pub(crate) mod grammar;
pub mod level;
pub mod moment;
pub mod morph;
pub mod morph_glyph;
pub mod once;
pub mod operation;
pub mod pending;
pub mod stamp;
pub mod touch;
pub mod tween;
pub mod use_detail;
pub mod use_operation;
pub mod use_pending;
