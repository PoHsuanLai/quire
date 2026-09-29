//! The grammar of small state details (design/26-DETAILS.md), enforced by types.
//!
//! A component keeps its state as its own enum and implements [`Detailed`] for it: the one place
//! that says what each transition means, as an exhaustive `match`. [`use_detail`] turns the state
//! into a [`Detail`], whose [`Cue`] is the only thing the primitives take, so a component cannot
//! play a moment its table does not name. A pending loop needs a [`PendingToken`] with a
//! deadline no later than `PendingCap`; an overshoot needs a [`Contact`], which only a pointer or
//! key handler's event can give; Reduced motion is inside every primitive, and every primitive's
//! clock stops when its moment ends.

pub mod armed;
pub mod check_mark;
pub mod count_up;
pub mod cue;
pub mod detailed;
pub mod first_show;
pub(crate) mod grammar;
pub mod layer_glyph;
pub mod level;
pub mod moment;
pub mod morph;
pub mod morph_glyph;
pub mod once;
pub mod operation;
pub mod pending;
pub mod reveal;
pub(crate) mod roll_digits;
pub mod settle;
pub mod stamp;
pub mod sweep;
pub mod touch;
pub mod tween;
pub mod use_detail;
pub mod use_operation;
pub mod use_pending;
pub mod use_settle;
