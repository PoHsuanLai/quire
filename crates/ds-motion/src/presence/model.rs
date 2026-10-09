//! A surface's life as a machine's state, inputs, outputs and settings.

use super::{Exit, Presence};
use ds_core::time::stamp::Stamp;
use ds_core::word::Word;
use ds_style::appearance::motion::MotionLevel;

use crate::anim::Anim;

/// Which of its entrance's two names a surface plays: flipped on each showing, so the entrance
/// restarts even where the engine kept the element's styles (design/05 section 9 rule 2). A
/// surface present again after its hide was taken back plays `hold` instead (`Held`): the exit
/// it drops is replaced by an animation that moves nothing, not by a second entrance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum EntranceAlias {
    A,
    B,
    Held,
}

impl EntranceAlias {
    pub(super) fn flipped(self) -> Self {
        match self {
            EntranceAlias::A => EntranceAlias::B,
            EntranceAlias::B | EntranceAlias::Held => EntranceAlias::A,
        }
    }
}

/// Where a surface is in its life, which entrance name it plays, and when the motion it is
/// playing settles. `until` is set exactly while `presence` is `Entering` or `Leaving`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Life {
    /// Its life: `data-presence` and `data-shown`.
    pub presence: Presence,
    /// Its entrance's name: `data-pulse`.
    pub alias: EntranceAlias,
    /// When the entrance or the exit it plays has settled.
    pub until: Option<Stamp>,
}

impl Life {
    /// A surface not drawn: before its first showing.
    pub fn hidden() -> Life {
        Life {
            presence: Presence::Hidden,
            alias: EntranceAlias::A,
            until: None,
        }
    }
}

/// What happened to the surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PresenceIn {
    /// The caller shows it.
    Show,
    /// The caller hides it.
    Hide,
    /// The motion it plays has settled (its wake came due).
    Elapsed,
}

impl From<ds_core::machine::Elapsed> for PresenceIn {
    fn from(_: ds_core::machine::Elapsed) -> Self {
        PresenceIn::Elapsed
    }
}

/// What the surface's owner does after a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PresenceOut {
    /// The exit has settled: tell the caller the surface has gone (`on_hidden`), so it can unmap it.
    Gone,
}

/// The animations a surface plays and the motion level that times them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PresenceParams {
    /// Played when it is shown.
    pub enter: Anim,
    /// Played when it is hidden.
    pub exit: Exit,
    /// The motion level the settle is measured at.
    pub motion: MotionLevel,
}
