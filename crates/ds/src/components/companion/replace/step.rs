//! The replacement's pure transition.

use super::model::{ReplaceIn, ReplaceOut, ReplacePhase};

/// The phase after `input`, and what it asks of its owner.
pub fn replace_step(_phase: ReplacePhase, _input: ReplaceIn) -> (ReplacePhase, Vec<ReplaceOut>) {
    todo!(
        "replace_step: design/32 section 4 and design/agent/ux.md section 3.4; replace_step_table pins every phase against every input"
    )
}
