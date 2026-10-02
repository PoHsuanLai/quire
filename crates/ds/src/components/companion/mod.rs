//! The companion's components (design/32 section 7): the orb that shows its presence, the chips a
//! prompt carries, the answer cards and what they hold, the plan list, the run rows, the
//! activity strip, the memory view and the served-by chip. Every one is a placeholder root until
//! its fill; the types are complete.
//!
//! The presentational marks an app reports (`ThingMark`, `ContextChip`, `FieldMode`, the summon
//! values) live in `ds-intents` and are re-exported here, once.

pub mod activity;
pub mod answer;
pub mod chips;
pub mod draft;
pub mod form;
pub mod mark;
pub mod memory;
pub mod orb;
pub mod plan;
pub mod port;
pub mod prompt;
pub mod proposed_event;
pub mod replace;
pub mod run_row;
pub mod served_by;

pub use ds_intents::{
    ChipKind, ContextChip, ContextModel, FieldMode, Removal, SummonAnswerMark, SummonSerial,
    ThingMark,
};
