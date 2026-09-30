//! The small-state details (design/26-DETAILS.md): moments, their grammar and the hooks that
//! play them.

pub use ds_motion::detail::{
    detailed::{first_table, moment_table},
    morph::{MorphStyle, Slashed},
    morph_glyph::MorphGlyph,
    once::use_shake,
    operation::{Operation, PendingToken},
    pending::{PendingLayers, PendingSpec, PendingStyle},
    stamp::EventStamp,
    touch::{Contact, Handled, Touch},
    use_detail::Detail,
    use_operation::use_operation,
    use_pending::use_pending,
};
