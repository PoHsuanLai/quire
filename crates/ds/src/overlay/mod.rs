//! Floating things render at the end of `.ds`, never inside the component that opened them:
//! `position:fixed` breaks inside transformed ancestors (design/05-MOTION.md section 9 rule 10).

pub(crate) mod host;
pub(crate) mod hover_hub;
pub(crate) mod menu_track;
pub(crate) mod pull_tab;
pub(crate) mod stack;
pub(crate) mod toast_hub;
