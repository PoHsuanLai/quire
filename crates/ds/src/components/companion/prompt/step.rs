//! The prompt's pure transition (design/32 section 4, the prompt state table).

use super::model::{PromptIn, PromptOut, PromptState};

/// The prompt after `input`, and what it asks of its field.
pub fn prompt_step(_state: PromptState, _input: PromptIn) -> (PromptState, Vec<PromptOut>) {
    todo!("prompt_step: the table in design/32 section 4; prompt_step_table pins every row")
}
