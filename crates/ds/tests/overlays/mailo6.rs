//! The mailo gaps 6 overlay case: an inline scrim with a layer of its own, above the positioned
//! rows its pane draws after it.

use crate::cases::Case;
use dioxus::prelude::*;
use ds::{Flow, Scrim, ZLayer};
use std::time::Duration;

pub const MAILO6_CASES: &[Case] = &[Case {
    component: "scrim",
    state: "inline-layer-raise",
    make: || {
        rsx! {
            div { style: "position:relative",
                Scrim { label: "Close peek", onclose: |_| {}, flow: Flow::Inline, layer: ZLayer::Raise }
                div { style: "position:relative", "A positioned row." }
            }
        }
    },
    wait: Duration::ZERO,
}];
