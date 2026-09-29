//! What a shown-and-hidden surface plays.

use super::exit::Exit;
use crate::motion::anim::Anim;

/// The animations of one surface's life: the entrance it plays when shown and the exit it plays
/// when hidden. `use_presence` times each by its settle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PresenceSpec {
    /// Played when it is shown.
    pub(crate) enter: Anim,
    /// Played when it is hidden.
    pub(crate) exit: Exit,
}
