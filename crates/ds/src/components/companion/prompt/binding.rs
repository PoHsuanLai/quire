//! A field's registration as a prompt target.

use super::host::PromptHost;
use super::model::PromptState;
use crate::components::fields::text_field_model::FieldKind;
use crate::focus::field::FieldHandle;
use dioxus::prelude::*;
use ds_intents::{ContextChip, FieldMode};

/// What a promptable field reads: its mode, the chips it carries and its prompt state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PromptBinding {
    /// Whether the field is an input or the prompt.
    pub mode: ReadSignal<FieldMode>,
    /// The chips the prompt carries.
    pub chips: ReadSignal<Vec<ContextChip>>,
    /// The prompt's state.
    pub state: ReadSignal<PromptState>,
    /// Where a summon would put the prompt in this field.
    pub host: PromptHost,
}

/// Registers `handle` as a promptable field of `kind`; `TextField` and `CommandPalette` call it
/// themselves.
pub fn use_prompt_target(_handle: FieldHandle, _kind: FieldKind) -> PromptBinding {
    todo!("use_prompt_target: registers the field with the CompanionPort and runs prompt_step")
}
