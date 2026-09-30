//! The pane switch as a pure machine: which of two panes is shown, and while
//! a switch plays, which one is arriving. The component feeds it the pane its caller asks for
//! and the settle of each round; every decision about what is drawn and what plays is here.

use crate::anim::Anim;
use ds_core::word::Word;

/// One of a switcher's two panes: the root (a control center's grid) or its detail (a module's
/// list).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum Pane {
    /// The pane the switcher starts from.
    #[default]
    Root,
    /// The pane a chevron opens.
    Detail,
}

impl Pane {
    /// The other pane.
    pub fn other(self) -> Pane {
        match self {
            Pane::Root => Pane::Detail,
            Pane::Detail => Pane::Root,
        }
    }

    /// What this pane plays arriving: the detail comes in from the right, the root from the
    /// left, as a push and its return.
    pub fn arrival(self) -> Anim {
        match self {
            Pane::Detail => Anim::PaneInR,
            Pane::Root => Anim::PaneInL,
        }
    }

    /// What this pane plays leaving: the other way from the pane replacing it.
    pub fn departure(self) -> Anim {
        match self {
            Pane::Root => Anim::PaneOutL,
            Pane::Detail => Anim::PaneOutR,
        }
    }
}

/// Which switch this is, counted from the switcher's mount: a settle names the round it ends,
/// so the settle of a switch that was reversed ends nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct PaneRound(pub u32);

/// Where a pane switch is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaneSlide {
    /// One pane shown, nothing moving.
    Rest(Pane),
    /// `to` arriving and the other pane leaving, in `round`.
    Moving {
        /// The pane arriving.
        to: Pane,
        /// The switch this is.
        round: PaneRound,
    },
}

/// What a pane is doing in a frame: how the component draws it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaneRole {
    /// Shown and still.
    Shown,
    /// Playing its arrival.
    Arriving,
    /// Playing its departure; drawn out of the flow so the height follows the arriving pane.
    Leaving,
    /// Not drawn.
    Absent,
}

impl PaneSlide {
    /// At rest on `pane`.
    pub fn rest(pane: Pane) -> Self {
        PaneSlide::Rest(pane)
    }

    /// The pane shown, or arriving.
    pub fn target(self) -> Pane {
        match self {
            PaneSlide::Rest(pane) | PaneSlide::Moving { to: pane, .. } => pane,
        }
    }

    /// What `pane` is doing now.
    pub fn role(self, pane: Pane) -> PaneRole {
        match self {
            PaneSlide::Rest(shown) if shown == pane => PaneRole::Shown,
            PaneSlide::Rest(_) => PaneRole::Absent,
            PaneSlide::Moving { to, .. } if to == pane => PaneRole::Arriving,
            PaneSlide::Moving { .. } => PaneRole::Leaving,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Pane;
    use crate::anim::Anim;

    #[test]
    fn each_pane_arrives_and_leaves_its_own_way() {
        const CASES: &[(Pane, Anim, Anim)] = &[
            (Pane::Detail, Anim::PaneInR, Anim::PaneOutR),
            (Pane::Root, Anim::PaneInL, Anim::PaneOutL),
        ];
        for &(pane, arrival, departure) in CASES {
            assert_eq!(pane.arrival(), arrival, "{pane:?}");
            assert_eq!(pane.departure(), departure, "{pane:?}");
            assert_eq!(pane.other().other(), pane);
        }
    }

    #[test]
    fn both_panes_play_at_one_duration_so_one_timer_settles_them() {
        for anim in [Anim::PaneInR, Anim::PaneInL, Anim::PaneOutL, Anim::PaneOutR] {
            assert_eq!(
                anim.recipe().duration,
                ds_style::tokens::timing::DurationToken::Move,
                "{anim:?}"
            );
        }
    }
}
