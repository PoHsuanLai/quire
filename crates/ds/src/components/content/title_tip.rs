//! A control's `title` as a Mac tooltip (`NSView.toolTip`, design/30 section 2.5): the control
//! feeds the hover hub from its own pointer events and a hooked [`Tooltip`] draws the tip, so
//! nothing wraps the control and its box, its siblings and its selectors are what they were.
//! The tip waits by the Tip profile (1 s cold, at once while the hub is warm), never takes focus,
//! never blocks a click and goes on a press.
//!
//! A control already inside a hint (a `Tooltip` the caller put round it) draws nothing: never two
//! tips. With no `Ds` to provide a hub the native `title` attribute stays.

use crate::host::measure::MountedRef;
use crate::stack::hover_hub::HoverKey;
use dioxus::core::current_scope_id;
use dioxus::prelude::*;

/// Context a hint provides to what it wraps: a control inside already has its tip.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Hinted;

/// The hover hub key a hint files its target under: its own, so two hints never share a card.
pub(crate) fn own_key() -> HoverKey {
    HoverKey(format!("hint:{}", current_scope_id().0))
}

/// What `Ds` lends a control to show its title, since the overlays that draw tips sit above
/// `controls`: the hub's three pointer verbs and the tip's surface. Provided by `overlays`.
#[derive(Clone, Copy)]
pub(crate) struct TipPort {
    /// The pointer came over the control, with its element once mounted.
    pub over: Callback<(HoverKey, Option<MountedRef>)>,
    /// The pointer left the control.
    pub out: Callback<()>,
    /// The control was pressed.
    pub press: Callback<()>,
    /// The tip's surface for `text`, keyed by the control's key.
    pub surface: fn(String, HoverKey) -> Element,
}

/// What a control does with its `title`.
#[derive(Clone)]
enum Say {
    /// Nothing: no title, or a tip of the caller's already stands over the control.
    Nothing,
    /// The plain `title` attribute: no `Ds` here provides a hub to draw a tip with.
    Native(String),
    /// A tooltip through the hover hub.
    Tip(String, TipPort),
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
    let port = try_consume_context::<TipPort>();
    let hinted = try_consume_context::<Hinted>().is_some();
    let say = match (title, port, hinted) {
        (None, _, _) | (Some(_), _, true) => Say::Nothing,
        (Some(text), Some(port), false) => Say::Tip(text, port),
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

    /// The value of `aria-description`: the tip's text, since a tip writes no `title` and a
    /// screen reader would not hear it (AXHelp). The native `title` serves itself; inside a
    /// hint the wrapper's text stands. Left out when it equals the accessible `name` (trimmed,
    /// case-sensitive): a screen reader would say the same words twice.
    pub(crate) fn description(&self, name: &str) -> Option<String> {
        match &self.say {
            Say::Tip(text, _) if text.trim() != name.trim() => Some(text.clone()),
            Say::Tip(..) | Say::Nothing | Say::Native(_) => None,
        }
    }

    /// The value of `data-tip`: the accessible `name`, written when a tip was given and
    /// `description` left it out for equalling that name. The name already says the tip, so a
    /// consumer's test cannot tell a titled button from an untitled one by `aria-description`;
    /// this marker can.
    pub(crate) fn marker(&self, name: &str) -> Option<String> {
        match (&self.say, self.description(name)) {
            (Say::Tip(..), None) => Some(name.to_owned()),
            _ => None,
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
        if let Say::Tip(_, port) = &self.say {
            event.stop_propagation();
            port.over
                .call((self.key.clone(), self.element.peek().clone()));
        }
    }

    /// The pointer left the control.
    pub(crate) fn out(&self) {
        if let Say::Tip(_, port) = &self.say {
            port.out.call(());
        }
    }

    /// A press removes the tip at once, not warm.
    pub(crate) fn press(&self) {
        if let Say::Tip(_, port) = &self.say {
            port.press.call(());
        }
    }

    /// The tip's surface, drawn beside the control (it takes no place in the layout).
    pub(crate) fn surface(&self) -> Element {
        match &self.say {
            Say::Tip(text, port) => (port.surface)(text.clone(), self.key.clone()),
            Say::Nothing | Say::Native(_) => rsx! {},
        }
    }
}
