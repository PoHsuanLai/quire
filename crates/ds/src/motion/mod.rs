//! Motion: every animation as data, the settle timers that replace `animationend`, and the
//! state machines that decide what moves (design/05-MOTION.md, design/06-INTERACTIONS.md).

pub mod anim;
pub(crate) mod batch_roster;
pub mod curve;
pub mod drag;
pub mod drag_return;
pub mod entrance;
pub mod hover_intent;
pub mod level_run;
pub mod pane_slide;
pub mod presence;
pub mod projection;
pub mod pulse;
pub(crate) mod pulse_key;
pub mod recipe;
pub(crate) mod recipe_detail;
pub(crate) mod recipe_own;
pub mod reduced;
pub mod roster;
pub(crate) mod roster_exits;
pub(crate) mod roster_rest;
pub mod settle;
pub mod spring;
pub mod spring_point;
pub mod spring_spec;
pub mod swipe;
pub mod timer;
pub mod use_level_run;
pub mod use_roster;
pub mod use_spring;
pub mod use_swipe;
pub mod velocity;
pub mod wake;

pub use crate::motion::recipe::{Fill, Iteration, Recipe};
pub use anim::Anim;
pub use drag::{DRAG_THRESHOLD, Drag, DragPhase, DragTracker, use_drag};
pub use drag_return::{DragReturn, use_drag_return};
pub use entrance::use_entrance;
pub use hover_intent::{HoverEvent, HoverIntent, IntentEffect, IntentPhase};
pub use level_run::{LevelRun, RunFrame, RunPhase, RunTail, RunTiming, RunTokens};
pub use pane_slide::{Pane, PaneRole, PaneRound, PaneSlide};
pub use presence::{Exit, ListPresence, Presence};
pub use projection::{DECELERATION_PER_MS, Throw};
pub use pulse::{Pulse, use_pulse};
pub use reduced::{FadeWay, ReducedForm};
pub use roster::{RosterEntry, RosterState, RowPitch, StayError, Stayed};
pub use settle::settle;
pub use spring::{Leg, Millis, Ratio, Spring, SpringPhase, State as SpringState};
pub use spring_point::{
    PointFrame, PointThrow, Release, SpringPointMotion, use_spring_point, use_spring_point_motion,
};
pub use spring_spec::{SpringResponse, SpringSpec};
pub use swipe::{
    Click, Speed, Stamp, SwipeEffect, SwipeInput, SwipeLook, SwipeMetrics, SwipeState,
};
pub use timer::{MotionTimer, TimerPhase, use_motion_timer};
pub use use_level_run::use_level_run;
pub use use_roster::{Roster, use_roster};
pub use use_spring::{PxPerUnit, SpringFrame, SpringMotion, use_spring, use_spring_motion};
pub use use_swipe::{Held, Swiper, use_swipe};
pub use velocity::{Velocity, VelocityMeter};
pub use wake::WakeStamp;
