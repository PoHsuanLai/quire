//! The memory view's data.

use crate::components::companion::answer::footer::SourceChip;
use ds_core::vocab::Tally;
use ds_core::word::Word;

/// Who a fact came from, `data-origin`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum FactOrigin {
    /// The person said it.
    YouSaid,
    /// The companion learned it.
    Companion,
    /// The nightly consolidation wrote it.
    Consolidated,
}

/// Where a fact stands, `data-standing`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum FactStanding {
    /// The person kept it, or it came from the person.
    Kept,
    /// It came from untrusted text and awaits the person.
    Pending,
    /// A newer fact replaced it.
    Superseded,
}

/// One row of the memory timeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryRowView {
    /// The memory item's key, opaque.
    pub key: String,
    /// The fact, in words.
    pub fact: String,
    /// When it was learned, in words.
    pub learned: String,
    /// The Space it belongs to.
    pub space: String,
    /// What it came from.
    pub sources: Vec<SourceChip>,
    /// Who it came from.
    pub origin: FactOrigin,
    /// Where it stands.
    pub standing: FactStanding,
}

/// A day of the timeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryDay {
    /// The day, in words.
    pub label: String,
    /// Its rows.
    pub rows: Vec<MemoryRowView>,
}

/// What forgetting a fact would also forget: "Also forgets 3 facts and 1 routine."
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForgetPreview {
    /// The fact.
    pub fact: String,
    /// How many facts derive from it.
    pub derived: Tally,
    /// How many routines derive from it.
    pub procedures: Tally,
}

/// One line of a consolidation diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffLine {
    /// A fact added.
    Added(String),
    /// Facts merged into one.
    Merged {
        /// The facts merged.
        from: Vec<String>,
        /// What they became.
        into: String,
    },
    /// A fact dropped.
    Dropped(String),
}

/// What one night's consolidation changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsolidationDiff {
    /// The night, in words.
    pub night: String,
    /// The changes.
    pub lines: Vec<DiffLine>,
}
