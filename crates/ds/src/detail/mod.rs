//! The grammar of small state details (design/26-DETAILS.md), enforced by types.
//!
//! A component keeps its state as its own enum and implements [`Detailed`] for it: the one place
//! that says what each transition means, as an exhaustive `match`. [`use_detail`] turns the state
//! into a [`Detail`], whose [`Cue`] is the only thing the primitives take, so a component cannot
//! play a moment its table does not name. A pending loop needs a [`PendingToken`] with a
//! deadline no later than `PendingCap`; an overshoot needs a [`Contact`], which only a pointer or
//! key handler's event can give; Reduced motion is inside every primitive, and every primitive's
//! clock stops when its moment ends.

mod count_up;
mod cue;
mod detailed;
mod first_show;
mod glide;
pub mod grammar;
mod level;
mod moment;
mod morph;
mod morph_glyph;
mod motor;
mod once;
mod operation;
mod pending;
mod reveal;
mod roll_digits;
mod settle;
mod stamp;
mod sweep;
mod touch;
mod tween;
mod use_detail;
mod use_operation;
mod use_pending;
mod use_settle;

pub use count_up::{CountPace, CountUp, use_count_up};
pub use cue::Cue;
pub use detailed::{Detailed, first_table, moment_table};
pub use first_show::FirstShow;
pub use moment::Moment;
pub use morph::{MorphStyle, Slashed};
pub use morph_glyph::MorphGlyph;
pub use once::{use_nudge, use_shake};
pub use operation::{Deadline, Operation, PendingToken};
pub use pending::{Layers, Lit, PendingFrame, PendingSpec, PendingStyle};
pub use reveal::Reveal;
pub use roll_digits::RollDigits;
pub use settle::{SettleStyle, Settling};
pub use stamp::EventStamp;
pub use sweep::{Sweep, use_sweep};
pub use touch::{Contact, Handled, Touch};
pub use tween::{Ease, Tween, TweenSpec, use_tween};
pub use use_detail::{Detail, use_detail};
pub use use_operation::use_operation;
pub use use_pending::use_pending;
pub use use_settle::use_settle;

/// The details' stylesheets, appended to the components' in cascade order.
pub(crate) const CSS: &[(&str, &str)] = &[
    ("detail_reveal", include_str!("reveal.css")),
    ("detail_morph", include_str!("morph.css")),
    ("detail_roll", include_str!("roll.css")),
];
