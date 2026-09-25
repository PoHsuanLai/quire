//! Motion: every animation as data, the settle timers that replace `animationend`, and the
//! state machines that decide what moves (design/05-MOTION.md, design/06-INTERACTIONS.md).

pub mod anim;
pub mod curve;
pub mod drag;
pub mod entrance;
pub mod hover_intent;
pub mod pane_slide;
pub mod presence;
pub mod pulse;
pub mod recipe;
mod recipe_own;
pub mod roster;
mod roster_rest;
pub mod settle;
pub mod timer;
pub mod use_roster;

pub use anim::{Anim, Fill, Iteration, Recipe};
pub use drag::{Drag, DragPhase, DragTracker, use_drag};
pub use entrance::use_entrance;
pub use hover_intent::{HoverEvent, HoverIntent, IntentEffect, IntentPhase};
pub use pane_slide::{Pane, PaneRole, PaneRound, PaneSlide};
pub use presence::{Exit, ListPresence, Presence};
pub use pulse::{Pulse, use_pulse};
pub use roster::{RosterEntry, RosterState, RowPitch, StayError, Stayed};
pub use settle::settle;
pub use timer::{MotionTimer, TimerPhase, use_motion_timer};
pub use use_roster::{Roster, use_roster};
