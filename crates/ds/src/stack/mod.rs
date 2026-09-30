//! Floating things render at the end of `.ds`, never inside the component that opened them:
//! `position:fixed` breaks inside transformed ancestors (design/05-MOTION.md section 9 rule 10).

pub mod host;
pub mod hover_hub;
pub mod layer_stack;
pub mod menu_track;
pub mod roving;
pub mod toast_hub;
pub mod typeahead;
