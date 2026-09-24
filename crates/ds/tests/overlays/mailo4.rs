//! The mailo gaps 4 overlay cases: hover cards keyed on the caller's own hooks through
//! `use_hover_intent`, drawn in place with no anchor or floating against a rect the caller
//! already had.

use crate::cases::Case;
use dioxus::prelude::*;
use ds::{
    Flow, HoverAnchor, HoverCard, HoverCardPart, HoverKey, HoverKind, Point, Px, Rect, Size,
    use_hover_intent,
};
use std::time::Duration;

/// Past the 450 ms hover intent.
const INTENT: Duration = Duration::from_millis(520);

pub const MAILO4_CASES: &[Case] = &[
    Case {
        component: "hover_card",
        state: "hook-keyed-inline",
        make: || rsx! { HookKeyed { anchor: HoverAnchor::Unplaced, flow: Flow::Inline } },
        wait: INTENT,
    },
    Case {
        component: "hover_card",
        state: "hook-keyed-unplaced",
        make: || rsx! { HookKeyed { anchor: HoverAnchor::Unplaced, flow: Flow::Floating } },
        wait: INTENT,
    },
    Case {
        component: "hover_card",
        state: "hook-keyed-rect",
        make: || rsx! { HookKeyed { anchor: HoverAnchor::Rect(name_rect()), flow: Flow::Floating } },
        wait: INTENT,
    },
];

/// Where a row's sender name was when the pointer came over it.
fn name_rect() -> Rect {
    Rect {
        origin: Point {
            x: Px(120.0),
            y: Px(40.0),
        },
        size: Size {
            width: Px(90.0),
            height: Px(18.0),
        },
    }
}

/// A sender card opened from the caller's own hook (here, as the page mounts), with no
/// target element of quire's anywhere.
#[component]
fn HookKeyed(anchor: HoverAnchor, flow: Flow) -> Element {
    let driver = use_hover_intent();
    use_hook(move || driver.over(HoverKey("sender:3".to_string()), HoverKind::Sender, anchor));
    let hub = driver.hub();
    let open = hub.open().or(hub.leaving());
    rsx! {
        div { class: "sender-slot",
            if let Some((key, kind)) = open {
                HoverCard {
                    key: "{key.0}",
                    kind,
                    flow,
                    parts: vec![
                        HoverCardPart::Title("Dana Okafor".to_string()),
                        HoverCardPart::Sub("dana@example.org".to_string()),
                    ],
                }
            }
        }
    }
}
