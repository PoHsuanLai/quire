//! The small-state details (design/26-DETAILS.md): moments, their grammar and the hooks that
//! play them.

pub use crate::motion::detail::{
    check_mark::CheckMark,
    count_up::{CountPace, use_count_up},
    cue::Cue,
    detailed::{Detailed, first_table, moment_table},
    first_show::FirstShow,
    layer_glyph::{LayerGlyph, Layering},
    moment::Moment,
    morph::{MorphStyle, Slashed},
    morph_glyph::MorphGlyph,
    once::{use_nudge, use_shake},
    operation::{Deadline, Operation, PendingToken},
    pending::{PendingLayers, PendingSpec, PendingStyle},
    reveal::{Reveal, RevealCue},
    roll_digits::RollDigits,
    settle::{SettleStyle, Settling},
    stamp::EventStamp,
    sweep::use_sweep,
    touch::{Contact, Handled, Touch},
    use_detail::{Detail, use_detail},
    use_operation::use_operation,
    use_pending::use_pending,
    use_settle::use_settle,
};
