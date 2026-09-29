//! The grammar of small state details (design/26-DETAILS.md), enforced by types.
//!
//! A component keeps its state as its own enum and implements [`Detailed`] for it: the one place
//! that says what each transition means, as an exhaustive `match`. [`use_detail`] turns the state
//! into a [`Detail`], whose [`Cue`] is the only thing the primitives take, so a component cannot
//! play a moment its table does not name. A pending loop needs a [`PendingToken`] with a
//! deadline no later than `PendingCap`; an overshoot needs a [`Contact`], which only a pointer or
//! key handler's event can give; Reduced motion is inside every primitive, and every primitive's
//! clock stops when its moment ends.

pub(crate) mod armed;
pub(crate) mod check_mark;
pub(crate) mod count_up;
pub(crate) mod cue;
pub(crate) mod detailed;
pub(crate) mod first_show;
pub(crate) mod glide;
pub(crate) mod grammar;
pub(crate) mod idle_dim;
pub(crate) mod layer_glyph;
pub(crate) mod level;
pub(crate) mod moment;
pub(crate) mod morph;
pub(crate) mod morph_glyph;
pub(crate) mod motor;
pub(crate) mod once;
pub(crate) mod operation;
pub(crate) mod pending;
pub(crate) mod reveal;
pub(crate) mod roll_digits;
pub(crate) mod settle;
pub(crate) mod stamp;
pub(crate) mod sweep;
pub(crate) mod touch;
pub(crate) mod tween;
pub(crate) mod use_detail;
pub(crate) mod use_operation;
pub(crate) mod use_pending;
pub(crate) mod use_settle;
