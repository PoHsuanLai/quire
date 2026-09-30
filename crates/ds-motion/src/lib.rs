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
pub mod long_press;
pub mod pane_slide;
pub mod presence;
pub mod projection;
pub(crate) mod pulse;
pub mod pulse_key;
pub mod recipe;
pub(crate) mod recipe_detail;
pub(crate) mod recipe_own;
pub(crate) mod reduced;
pub mod roster;
pub(crate) mod roster_rest;
pub mod rubber;
pub mod settle;
pub mod spring;
pub mod spring_point;
pub mod spring_spec;
pub mod swipe;
pub mod timeline;
pub mod timer;
pub mod use_collapse;
pub mod use_roster;
pub mod use_spring;
pub mod use_swipe;
pub mod velocity;
pub mod wake;
