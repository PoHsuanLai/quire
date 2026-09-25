//! Where an item is in its life: entering, present, leaving by some exit, or healing a gap
//! (design/04-COMPONENTS.md "Motion states", design/05-MOTION.md section 8).

use crate::components::vocab::StaggerIndex;
use crate::geometry::units::Px;

/// How a row leaves: `data-exit`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Exit {
    /// Archive, restore, wake: `fold`.
    Fold,
    /// Snooze: `curl`.
    Curl,
    /// Trash, delete: `crumple`.
    Crumple,
    /// A Today entry closing: `tab-out` (wave 2 integration amendment).
    TabOut,
    /// A notification banner leaving: `banner-out`, a slide to the right (sill Q121).
    BannerOut,
}

impl Exit {
    /// The `data-exit` value.
    pub fn slug(self) -> &'static str {
        match self {
            Exit::Fold => "fold",
            Exit::Curl => "curl",
            Exit::Crumple => "crumple",
            Exit::TabOut => "tab-out",
            Exit::BannerOut => "banner-out",
        }
    }
}

/// An item's motion state: `data-presence`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Presence {
    /// Playing its entrance.
    Entering,
    /// At rest.
    Present,
    /// Playing an exit; dropped when it settles.
    Leaving(Exit),
    /// Sliding up by `dy` into the gap a removed row left, delayed by `d` heal steps.
    Healing {
        /// How far it starts below its resting place.
        dy: Px,
        /// Rows counted from the removed one: the heal delay's multiplier.
        d: StaggerIndex,
    },
}

/// Whether a whole list is playing its first-show entrance: `.ds-list[data-presence]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ListPresence {
    /// First shown: rows rise, staggered.
    Entering,
    /// At rest: re-renders do not replay the entrance.
    #[default]
    Present,
}

impl Presence {
    /// The `data-presence` value.
    pub fn slug(self) -> &'static str {
        match self {
            Presence::Entering => "entering",
            Presence::Present => "present",
            Presence::Leaving(_) => "leaving",
            Presence::Healing { .. } => "healing",
        }
    }
}
