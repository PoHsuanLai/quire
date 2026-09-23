//! The undo toast's pull tab as a pure machine: drag right past the arm point, or click
//! without moving, to undo (design/06-INTERACTIONS.md section 9.2, design/04-COMPONENTS.md
//! section 23).

use crate::geometry::units::Px;

/// How far left the tab may be pulled.
const PULL_MIN: Px = Px(-6.0);
/// How far right the tab may be pulled.
const PULL_MAX: Px = Px(78.0);
/// The tab is armed once pulled further right than this.
const ARM_AT: Px = Px(46.0);
/// A click moved less than this is a tap, and undoes.
const TAP_SLOP: Px = Px(3.0);

/// Whether a release would undo: `data-armed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TabArm {
    /// Pulled past the arm point: the accent fill, and a release undoes.
    Armed,
    /// Not far enough.
    #[default]
    Disarmed,
}

impl TabArm {
    /// The `data-armed` value.
    pub fn slug(self) -> &'static str {
        match self {
            TabArm::Armed => "armed",
            TabArm::Disarmed => "disarmed",
        }
    }
}

/// What a release or a click on the tab asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pull {
    /// Undo: call `ToastHub::undo`.
    Undo,
    /// Nothing: the tab springs back.
    Hold,
}

/// Where the tab is.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PullPhase {
    /// At rest.
    #[default]
    Idle,
    /// Held down at `x0`, pulled `dx` (already clamped).
    Dragging {
        /// Where the pointer went down.
        x0: Px,
        /// The clamped pull.
        dx: Px,
    },
}

/// The pull tab machine: its phase, plus how far the last drag moved, which the click that
/// follows a release reads.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PullTab {
    phase: PullPhase,
    last: Px,
}

impl PullTab {
    /// Pointer down on the tab at `x`.
    pub fn down(self, x: Px) -> Self {
        PullTab {
            phase: PullPhase::Dragging { x0: x, dx: Px(0.0) },
            last: Px(0.0),
        }
    }

    /// The pointer moved to `x`.
    pub fn moved(self, x: Px) -> Self {
        match self.phase {
            PullPhase::Dragging { x0, .. } => PullTab {
                phase: PullPhase::Dragging {
                    x0,
                    dx: Px((x - x0).0.clamp(PULL_MIN.0, PULL_MAX.0)),
                },
                ..self
            },
            PullPhase::Idle => self,
        }
    }

    /// Pointer up or cancelled: undo when released armed. The pull is kept for the click
    /// that follows, so a drag that ends on the tab is not also read as a tap.
    pub fn up(self) -> (Self, Pull) {
        match self.phase {
            PullPhase::Dragging { dx, .. } => (
                PullTab {
                    phase: PullPhase::Idle,
                    last: dx,
                },
                if dx.0 > ARM_AT.0 {
                    Pull::Undo
                } else {
                    Pull::Hold
                },
            ),
            PullPhase::Idle => (self, Pull::Hold),
        }
    }

    /// A click on the tab: a tap (the last drag moved under 3 px) undoes.
    pub fn click(self) -> (Self, Pull) {
        let pull = if self.last.0.abs() < TAP_SLOP.0 {
            Pull::Undo
        } else {
            Pull::Hold
        };
        (
            PullTab {
                phase: PullPhase::Idle,
                last: Px(0.0),
            },
            pull,
        )
    }

    /// Where the tab is.
    pub fn phase(&self) -> PullPhase {
        self.phase
    }

    /// The tab's `translateX`: the pull while dragging, else zero (it springs back).
    pub fn dx(&self) -> Px {
        match self.phase {
            PullPhase::Dragging { dx, .. } => dx,
            PullPhase::Idle => Px(0.0),
        }
    }

    /// Whether a release now would undo.
    pub fn arm(&self) -> TabArm {
        if self.dx().0 > ARM_AT.0 {
            TabArm::Armed
        } else {
            TabArm::Disarmed
        }
    }
}
