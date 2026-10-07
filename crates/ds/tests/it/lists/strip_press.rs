//! The mail-app list cases: a strip whose press is heard before any measurement (its markup
//! is the strip's own: the press is a listener, which a server render does not write).

use super::cases::Case;
use super::rows::strip_actions;
use dioxus::prelude::*;
use ds::components::app::hover_strip::{ActionId, HoverStrip};
use ds::prelude::*;

pub const STRIP_PRESS_CASES: &[Case] = &[Case {
    component: "hover_strip",
    state: "on-press",
    make: || rsx! { HoverStrip { actions: strip_actions(), shown: Shown::Visible, on_press: |_: ActionId| {} } },
}];
