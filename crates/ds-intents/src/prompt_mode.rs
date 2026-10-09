//! Whether a field takes input or a prompt.

use ds_core::word::Word;

/// A field's mode (`data-mode`). `Input` is the field as it has always been; `Prompt` is the
/// same field taken over by a summon, its query kept for the way back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
#[non_exhaustive]
pub enum FieldMode {
    /// A field for input.
    #[default]
    Input,
    /// A field that is the companion's prompt.
    Prompt,
}
