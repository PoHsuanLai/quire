//! The words a `ModuleTile` is described in: whether the module is on, whether it has a detail
//! pane, and how many grid columns it takes (sill FINDINGS Q78).

use crate::detail::{Detailed, Moment};

/// Where a module is: off, on, or on its way (connecting, scanning).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ModuleState {
    /// Off: a paper disc with ink.
    #[default]
    Off,
    /// On: the disc in `--accent`, the tile's plate in `--accent-soft`.
    On,
    /// Working towards a state the module has not reached: the Spinner's breathe on the disc.
    Busy,
}

impl ModuleState {
    /// The `data-state` word.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            ModuleState::Off => "off",
            ModuleState::On => "on",
            ModuleState::Busy => "busy",
        }
    }

    /// `aria-pressed`: a busy module is neither on nor off yet, which ARIA calls mixed.
    pub(crate) fn aria_pressed(self) -> &'static str {
        match self {
            ModuleState::Off => "false",
            ModuleState::On => "true",
            ModuleState::Busy => "mixed",
        }
    }

    /// `aria-busy`: written only while busy.
    pub(crate) fn aria_busy(self) -> Option<&'static str> {
        match self {
            ModuleState::Busy => Some("true"),
            ModuleState::Off | ModuleState::On => None,
        }
    }
}

impl Detailed for ModuleState {
    /// Turning busy starts an operation (Pending, R4); busy ending on is its success; every other
    /// change is a Change (a busy module that falls back off is not claimed as a failure: the
    /// module's own words say why).
    fn moment(from: &Self, to: &Self) -> Moment {
        match (from, to) {
            (ModuleState::Off | ModuleState::On | ModuleState::Busy, ModuleState::Busy) => {
                Moment::Pending
            }
            (ModuleState::Busy, ModuleState::On) => Moment::Success,
            (ModuleState::Off | ModuleState::On, ModuleState::On)
            | (ModuleState::Off | ModuleState::On | ModuleState::Busy, ModuleState::Off) => {
                Moment::Change
            }
        }
    }

    /// A tile shown busy is an operation already running; any other state is simply there.
    fn first(state: &Self) -> Moment {
        match state {
            ModuleState::Busy => Moment::Pending,
            ModuleState::Off | ModuleState::On => Moment::Rest,
        }
    }
}

/// Whether a tile ends in a chevron that opens the module's detail pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Chevron {
    /// No detail pane: the tile only toggles.
    #[default]
    None,
    /// A chevron, its own hit target, opening the detail pane.
    Detail,
}

/// How many of the control center's grid columns a tile takes (design/13 section 13.3.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TileSpan {
    /// One column.
    #[default]
    Half,
    /// Every column (`grid-column:1 / -1`), as a slider module.
    Full,
}

impl TileSpan {
    /// The `data-span` word.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            TileSpan::Half => "half",
            TileSpan::Full => "full",
        }
    }
}
