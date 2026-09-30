//! The small-state details (design/26-DETAILS.md): moments, their grammar and the hooks that
//! play them.

pub use ds_motion::detail::{
    cue::Cue,
    detailed::{Detailed, first_table, moment_table},
    moment::Moment,
    morph::{MorphStyle, Slashed},
    morph_glyph::MorphGlyph,
    once::use_shake,
    operation::{Operation, PendingToken},
    pending::{PendingLayers, PendingSpec, PendingStyle},
    stamp::EventStamp,
    touch::{Contact, Handled, Touch},
    use_detail::{Detail, use_detail},
    use_operation::use_operation,
    use_pending::use_pending,
};
