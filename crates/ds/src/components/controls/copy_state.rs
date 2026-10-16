//! Whether a copy has happened, as the host knows it.

use ds_core::word::Word;

/// Whether the text a control offers to copy has been copied, as the host knows it: the host
/// writes the clipboard (this crate is renderer-free) and says so, and the control's label follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum CopyState {
    /// Nothing copied lately.
    #[default]
    Idle,
    /// The host just copied it: the button says so.
    Copied,
}
