//! What a shown-and-hidden surface plays.

use super::exit::Exit;
use crate::anim::Anim;

/// The animations of one surface's life: the entrance it plays when shown and the exit it plays
/// when hidden. `use_presence` times each by its settle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PresenceSpec {
    /// Played when it is shown.
    pub enter: Anim,
    /// Played when it is hidden.
    pub exit: Exit,
}
