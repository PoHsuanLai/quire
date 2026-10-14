//! The answer card and what every card shares (design/32 section 4): the actions, the footer and
//! the answer variants without a card of their own. The card draws every variant's body itself;
//! the plan and the replacement have their own lists and bars for where they stand alone.

pub mod action;
pub mod actions;
pub mod built_in;
pub mod draft_body;
pub mod event_body;
pub mod footer;
pub mod footer_view;
pub mod form_body;
pub mod model;
pub mod person;
pub mod plan_body;
pub mod refused;
pub mod refused_body;
pub mod replace_body;
pub mod text_body;
pub mod view;
