//! The port `Ds` lends every control so its `title` shows as a tooltip (design/30 section 2.5):
//! the hub's pointer verbs and the tooltip's surface, which sit above `controls` and so are
//! handed down as context (see `controls::button_tip`).

use crate::components::controls::button_tip::TipPort;
use crate::components::overlays::hover_card::intent::{HoverAnchor, use_hover_intent};
use crate::components::overlays::tooltip::Tooltip;
use crate::stack::hover_hub::HoverKey;
use dioxus::prelude::*;
use ds_motion::hover_intent::HoverProfile;

/// The tip's surface: a hooked tooltip, keyed by the control's own key.
fn surface(text: String, key: HoverKey) -> Element {
    rsx! {
        Tooltip { text, hover_key: Some(key) }
    }
}

/// Provide the port to everything under the caller. Needs the hover hub provided already.
pub fn use_title_tips_provider() {
    let driver = use_hover_intent();
    let over = use_callback(move |(key, element): (HoverKey, Option<_>)| {
        let anchor = element.map_or(HoverAnchor::Unplaced, HoverAnchor::Element);
        driver.over(key, HoverProfile::Tip, anchor);
    });
    let out = use_callback(move |()| driver.out());
    let press = use_callback(move |()| driver.press());
    use_context_provider(|| TipPort {
        over,
        out,
        press,
        surface,
    });
}
