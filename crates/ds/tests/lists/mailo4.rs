//! The mailo gaps 4 list cases: a strip whose press is heard before any measurement (its markup
//! is the strip's own: the press is a listener, which a server render does not write).

use crate::cases::Case;
use crate::rows::strip_actions;
use dioxus::prelude::*;
use ds::{ActionId, HoverStrip, Shown};

pub const MAILO4_CASES: &[Case] = &[Case {
    component: "hover_strip",
    state: "on-press",
    make: || rsx! { HoverStrip { actions: strip_actions(), shown: Shown::Visible, on_press: |_: ActionId| {} } },
}];
