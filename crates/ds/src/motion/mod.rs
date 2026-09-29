//! Motion: every animation as data, the settle timers that replace `animationend`, and the
//! state machines that decide what moves (design/05-MOTION.md, design/06-INTERACTIONS.md).

pub(crate) mod anim;
pub(crate) mod batch_roster;
pub(crate) mod css;
pub(crate) mod detail;
pub(crate) mod drag;
pub(crate) mod drag_return;
pub(crate) mod entrance;
pub(crate) mod hover_intent;
pub(crate) mod kit;
pub(crate) mod level_run;
pub(crate) mod pane_slide;
pub(crate) mod presence;
pub(crate) mod projection;
pub(crate) mod pulse;
pub(crate) mod pulse_key;
pub(crate) mod recipe;
pub(crate) mod recipe_detail;
pub(crate) mod recipe_own;
pub(crate) mod reduced;
pub(crate) mod roster;
pub(crate) mod roster_exits;
pub(crate) mod roster_rest;
pub(crate) mod settle;
pub(crate) mod spring;
pub(crate) mod spring_point;
pub(crate) mod spring_spec;
pub(crate) mod swipe;
pub(crate) mod timer;
pub(crate) mod use_level_run;
pub(crate) mod use_roster;
pub(crate) mod use_spring;
pub(crate) mod use_swipe;
pub(crate) mod velocity;
pub(crate) mod wake;

pub use crate::motion::{
    drag::fraction_along,
    drag_return::{DragReturn, use_drag_return},
    projection::Throw,
    spring::{Ratio, SpringPhase},
    spring_point::{
        PointThrow, Release, SpringPointMotion, use_spring_point, use_spring_point_motion,
    },
    spring_spec::{SpringResponse, SpringSpec},
    use_spring::{PxPerUnit, use_spring},
    velocity::{Velocity, VelocityMeter},
};
