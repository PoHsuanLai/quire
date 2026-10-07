//! A control's `title` as a Mac tooltip (`NSView.toolTip`, design/30 section 2.5): the control
//! feeds the hover hub from its own pointer events and a hooked [`Tooltip`] draws the tip, so
//! nothing wraps the control and its box, its siblings and its selectors are what they were.
//! The tip waits by the Tip profile (1 s cold, at once while the hub is warm), never takes focus,
//! never blocks a click and goes on a press.
//!
//! A control already inside a hint (a `Tooltip` the caller put round it) draws nothing: never two
//! tips. With no `Ds` to provide a hub the native `title` attribute stays.

use crate::components::overlays::hover_card::intent::{
    HoverAnchor, HoverDriver, try_use_hover_intent,
};
use crate::components::overlays::tooltip::{Hinted, Tooltip, own_key};
use crate::host::measure::MountedRef;
use crate::stack::hover_hub::HoverKey;
use dioxus::prelude::*;
use ds_motion::hover_intent::HoverProfile;

/// What a control does with its `title`.
#[derive(Clone)]
enum Say {
    /// Nothing: no title, or a tip of the caller's already stands over the control.
    Nothing,
    /// The plain `title` attribute: no `Ds` here provides a hub to draw a tip with.
    Native(String),
    /// A tooltip through the hover hub.
    Tip(String, HoverDriver),
}

/// A control's title and the handles that show it. Built once per render; cloning it is cheap.
#[derive(Clone)]
pub(crate) struct Tip {
    say: Say,
    key: HoverKey,
    element: Signal<Option<MountedRef>>,
}

/// How `title` is shown here. Called unconditionally, every render, as it holds hooks.
pub(crate) fn use_tip(title: Option<String>) -> Tip {
    let key = use_hook(own_key);
    let element = use_signal(|| None::<MountedRef>);
    let driver = try_use_hover_intent();
    let hinted = try_consume_context::<Hinted>().is_some();
    let say = match (title, driver, hinted) {
        (None, _, _) | (Some(_), _, true) => Say::Nothing,
        (Some(text), Some(driver), false) => Say::Tip(text, driver),
        (Some(text), None, false) => Say::Native(text),
    };
    Tip { say, key, element }
}

impl Tip {
    /// The value of the control's `title` attribute.
    pub(crate) fn native(&self) -> Option<String> {
        match &self.say {
            Say::Native(text) => Some(text.clone()),
            Say::Nothing | Say::Tip(..) => None,
        }
    }

    /// The control's element mounted: kept, to place the tip below it.
    pub(crate) fn mounted(&self, event: &MountedEvent) {
        let mut element = self.element;
        element.set(Some(MountedRef(event.data())));
    }

    /// The pointer came over the control. The innermost control wins, as a hover target does: it
    /// handles the pointer and stops it.
    pub(crate) fn over(&self, event: &MouseEvent) {
        if let Say::Tip(_, driver) = &self.say {
            event.stop_propagation();
            let anchor = self
                .element
                .peek()
                .clone()
                .map_or(HoverAnchor::Unplaced, HoverAnchor::Element);
            driver.over(self.key.clone(), HoverProfile::Tip, anchor);
        }
    }

    /// The pointer left the control.
    pub(crate) fn out(&self) {
        if let Say::Tip(_, driver) = &self.say {
            driver.out();
        }
    }

    /// A press removes the tip at once, not warm.
    pub(crate) fn press(&self) {
        if let Say::Tip(_, driver) = &self.say {
            driver.press();
        }
    }

    /// The tip's surface, drawn beside the control (it takes no place in the layout).
    pub(crate) fn surface(&self) -> Element {
        match &self.say {
            Say::Tip(text, _) => rsx! {
                Tooltip { text: text.clone(), hover_key: Some(self.key.clone()) }
            },
            Say::Nothing | Say::Native(_) => rsx! {},
        }
    }
}
