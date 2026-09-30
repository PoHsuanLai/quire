//! A tooltip keyed by the caller's own pointer hooks (design/30 section 2.5): a thread row's time
//! names its full date on hover, the row's own events feeding the hover hub.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::components::overlays::hover_card::intent::{HoverAnchor, use_hover_intent};
use ds::host::measure::MountedRef;
use ds::motion::hover_intent::HoverProfile;
use ds::prelude::*;
use ds::stack::hover_hub::HoverKey;

/// Two times, each with a tip that opens from the time's own pointer events.
#[component]
pub fn TipHooks() -> Element {
    rsx! {
        Section {
            title: "Tooltip: keyed by the caller's hooks",
            note: "Where the caller already has the pointer (a thread row's time), it feeds use_hover_intent from its own events and draws a Tooltip keyed by it: nothing is wrapped, the tip opens after the Tip delay below the element it filed and goes when the pointer leaves. Rest on a time.",
            div { class: "g-row",
                Time { at: 1, shown: "3:42 PM", full: "Thursday, October 1, 2026 at 3:42 PM" }
                Time { at: 2, shown: "Sep 28", full: "Monday, September 28, 2026 at 9:05 AM" }
            }
        }
    }
}

/// One time with its tip.
#[component]
fn Time(at: u32, shown: &'static str, full: &'static str) -> Element {
    let driver = use_hover_intent();
    let mut element = use_signal(|| None::<MountedRef>);
    let key = HoverKey(format!("gallery-time:{at}"));
    let tip = key.clone();
    rsx! {
        span {
            class: "g-code",
            onmounted: move |event| element.set(Some(MountedRef(event.data()))),
            onpointerenter: move |_| {
                let anchor = element
                    .peek()
                    .clone()
                    .map_or(HoverAnchor::Unplaced, HoverAnchor::Element);
                driver.over(key.clone(), HoverProfile::Tip, anchor);
            },
            onpointerleave: move |_| driver.out(),
            "{shown}"
        }
        Tooltip { text: full, hover_key: Some(tip) }
    }
}
