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
    /// The file name without its extension selected, so typing replaces "photo" of "photo.png".
    Stem,
}

impl From<Select> for Option<InitialCaret> {
    /// The caret a focus write puts: [`Select::All`] is [`InitialCaret::SelectAll`],
    /// [`Select::Stem`] is [`InitialCaret::SelectStem`], and
    /// [`Select::None`] leaves the caret where the renderer puts it.
    fn from(select: Select) -> Self {
        match select {
            Select::None => None,
            Select::All => Some(InitialCaret::SelectAll),
            Select::Stem => Some(InitialCaret::SelectStem),
        }
    }
}
