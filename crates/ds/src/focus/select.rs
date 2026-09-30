//! What a focus change does with a field's text once the caret is in it: a
//! rename field opened on a folder's name selects it, so typing replaces the name. Blitz has no
//! script to call `input.select()` with, so the host does it
//! ([`FocusHost::select`](crate::host::parts::FocusHost::select)), after its focus write has landed.

use crate::host::caret::InitialCaret;

/// The field's text after the focus lands in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Select {
    /// Left as it is: the caret where the renderer puts it.
    #[default]
    None,
    /// All of it selected, so the first key typed replaces it.
    All,
}

/// What a focus write does with the field's text once the caret is in it: nothing, or the caret
/// put at an [`InitialCaret`] place (a [`Select::All`] is [`InitialCaret::SelectAll`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Landing {
    /// Left where the renderer puts it.
    Leave,
    /// Put there.
    Place(InitialCaret),
}

impl From<Select> for Landing {
    fn from(select: Select) -> Self {
        match select {
            Select::None => Landing::Leave,
            Select::All => Landing::Place(InitialCaret::SelectAll),
        }
    }
}
