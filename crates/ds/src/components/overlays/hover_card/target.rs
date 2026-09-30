//! HoverTarget: what a card hooks, feeding the hover hub (design/06-INTERACTIONS.md section 3).
//!
//! The target wraps its children in an element of its own, carrying the hub's handlers. A
//! `span` cannot hold a list item, so the caller picks the element: a `span`
//! (the default, inline), a `div`, or an `li` that is itself the list's item. The handlers are
//! the same on each, so the hover-intent machine sees one kind of target whatever it is drawn
//! as. A wrapper with no box of its own (`display:contents`) is not offered: the card is placed
//! against the target's measured rect, and such an element has none.

use super::intent::{HoverAnchor, HoverDriver, use_hover_intent};
use super::kind_slug;
use crate::host::measure::MountedRef;
use crate::stack::hover_hub::{HoverKey, HoverKind};
use dioxus::prelude::*;
use ds_motion::hover_intent::HoverProfile;

/// The element a hover target is drawn as.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TargetElement {
    /// An inline `span`: a name, a time.
    #[default]
    Span,
    /// A block `div`.
    Div,
    /// An `li`, the list's own item: a sidebar entry, a person in a list.
    Li,
}

/// One target's handlers, shared by whichever element draws it.
#[derive(Clone)]
struct Hooks {
    driver: HoverDriver,
    element: Signal<Option<MountedRef>>,
    key: HoverKey,
    profile: HoverProfile,
}

impl Hooks {
    /// The target's element mounted: kept, to measure where the card goes.
    fn mounted(&self, event: MountedEvent) {
        let mut element = self.element;
        element.set(Some(MountedRef(event.data())));
    }

    /// The pointer came over the target. The innermost target wins (`S:1787-1788`): it handles
    /// the pointer and stops it.
    fn over(&self, event: MouseEvent) {
        event.stop_propagation();
        let anchor = self
            .element
            .peek()
            .clone()
            .map_or(HoverAnchor::Unplaced, HoverAnchor::Element);
        self.driver.over(self.key.clone(), self.profile, anchor);
    }

    /// The pointer left the target.
    fn out(&self) {
        self.driver.out();
    }

    /// A click removes the card at once, not warm (`S:1809`).
    fn down(&self) {
        self.driver.press();
    }
}

/// Wraps whatever a hover interface hooks, drawn as `as_`: feeds the hover hub, which opens the
/// card, tooltip or label after `profile`'s delay.
///
/// `kind` is a card's content kind (`data-kind`); a tooltip or a label has none.
///
/// The component doc names the first prop `key`; dioxus reserves `key` for list identity and
/// rejects a prop of that name, so it is `hover_key` (FINDINGS.md).
#[component]
pub fn HoverTarget(
    hover_key: HoverKey,
    #[props(default, into)] kind: Option<HoverKind>,
    #[props(default)] profile: HoverProfile,
    #[props(default)] as_: TargetElement,
    children: Element,
) -> Element {
    let hooks = Hooks {
        driver: use_hover_intent(),
        element: use_signal(|| None::<MountedRef>),
        key: hover_key.clone(),
        profile,
    };
    let (mount, over, out, down) = (hooks.clone(), hooks.clone(), hooks.clone(), hooks);
    let slug = kind.map(kind_slug);
    // A hint (tooltip, label) wraps a control, so it is a flex box that is as big as the control.
    let hint = (profile != HoverProfile::Card).then_some("true");
    match as_ {
        TargetElement::Span => rsx! {
            span {
                class: "ds-hover-target",
                "data-hover-key": "{hover_key.0}",
                "data-kind": slug,
                "data-hint": hint,
                onmounted: move |event| mount.mounted(event),
                onmouseover: move |event| over.over(event),
                onmouseleave: move |_| out.out(),
                onpointerdown: move |_| down.down(),
                {children}
            }
        },
        TargetElement::Div => rsx! {
            div {
                class: "ds-hover-target",
                "data-hover-key": "{hover_key.0}",
                "data-kind": slug,
                "data-hint": hint,
                "data-as": "div",
                onmounted: move |event| mount.mounted(event),
                onmouseover: move |event| over.over(event),
                onmouseleave: move |_| out.out(),
                onpointerdown: move |_| down.down(),
                {children}
            }
        },
        TargetElement::Li => rsx! {
            li {
                class: "ds-hover-target",
                "data-hover-key": "{hover_key.0}",
                "data-kind": slug,
                "data-hint": hint,
                "data-as": "li",
                onmounted: move |event| mount.mounted(event),
                onmouseover: move |event| over.over(event),
                onmouseleave: move |_| out.out(),
                onpointerdown: move |_| down.down(),
                {children}
            }
        },
    }
}
