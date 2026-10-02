//! Where a summon in a field of a given kind puts the prompt.

use crate::components::fields::text_field_model::FieldKind;
use ds_core::word::Word;

/// Where a summon in a field of this kind puts the prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum PromptHost {
    /// A plain or search field: the field itself becomes the prompt.
    Field,
    /// A multiline field or the editor: a prompt anchored at the selection, whose answer is an
    /// inline replace.
    Anchored,
    /// A secure field: never; the summon falls through to the launcher with no field context.
    Refused,
}

/// Where a summon in a field of `kind` puts the prompt. Total.
pub fn prompt_host(kind: FieldKind) -> PromptHost {
    match kind {
        FieldKind::Plain | FieldKind::Search => PromptHost::Field,
        FieldKind::Multiline => PromptHost::Anchored,
        FieldKind::Secure => PromptHost::Refused,
    }
}
