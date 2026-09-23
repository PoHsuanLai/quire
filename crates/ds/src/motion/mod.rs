//! Motion: every animation as data, the settle timers that replace `animationend`, and the
//! state machines that decide what moves (design/05-MOTION.md, design/06-INTERACTIONS.md).

pub mod anim;
pub mod drag;
pub mod hover_intent;
pub mod presence;
pub mod pulse;
pub mod recipe;
pub mod roster;
pub mod settle;
pub mod timer;
pub mod use_roster;

pub use anim::{Anim, Fill, Iteration, Recipe};
pub use drag::{Drag, DragPhase, DragTracker, use_drag};
pub use hover_intent::{HoverEvent, HoverIntent, IntentEffect, IntentPhase};
pub use presence::{Exit, ListPresence, Presence};
pub use pulse::{Pulse, use_pulse};
pub use roster::{RosterEntry, RosterState, RowPitch};
pub use settle::settle;
pub use timer::{MotionTimer, TimerPhase, use_motion_timer};
pub use use_roster::{Roster, use_roster};
