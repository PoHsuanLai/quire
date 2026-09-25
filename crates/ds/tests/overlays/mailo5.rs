//! The mailo gaps 5 overlay cases: a scrim drawn inline in the pane it dims, under the reader
//! the pane draws after it.

use crate::cases::Case;
use dioxus::prelude::*;
use ds::{Flow, Scrim};
use std::time::Duration;

const NOW: Duration = Duration::ZERO;

pub const MAILO5_CASES: &[Case] = &[Case {
    component: "scrim",
    state: "inline",
    make: || {
        rsx! {
            div { class: "pane", style: "position:relative",
                p { "The list." }
                Scrim { label: "Close peek", onclose: |_| {}, flow: Flow::Inline }
                article { "The peeked reader." }
            }
        }
    },
    wait: NOW,
}];
