//! Motion: every animation as data, the settle timers that replace `animationend`, and the
//! state machines that decide what moves (design/05-MOTION.md, design/06-INTERACTIONS.md).

pub mod anim;
pub(crate) mod css;
pub mod detail;
pub mod drag;
pub mod drag_return;
pub mod entrance;
pub mod hover_intent;
pub mod kit;
pub mod pane_slide;
pub mod presence;
pub mod projection;
pub(crate) mod pulse;
pub mod pulse_key;
pub(crate) mod recipe;
pub(crate) mod recipe_detail;
pub(crate) mod recipe_own;
pub(crate) mod reduced;
pub mod roster;
pub(crate) mod roster_rest;
pub mod settle;
pub mod spring;
pub(crate) mod spring_point;
pub mod spring_spec;
pub mod swipe;
pub mod timeline;
pub mod timer;
pub mod use_roster;
pub mod use_spring;
pub mod use_swipe;
pub mod velocity;
pub mod wake;

pub use crate::motion::{
    drag::fraction_along,
    drag_return::{DragReturn, use_drag_return},
    projection::Throw,
    spring::{Ratio, SpringPhase},
    spring_point::{
        PointThrow, Release, SpringPointMotion, use_spring_point, use_spring_point_motion,
    },
    spring_spec::{SpringResponse, SpringSpec},
    timeline::spring::PxPerUnit,
    use_spring::use_spring,
    velocity::{Velocity, VelocityMeter},
};
