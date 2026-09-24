//! HoverTarget: what a card hooks, feeding the hover hub (design/06-INTERACTIONS.md section 3).
//!
//! The target wraps its children in an element of its own, carrying the hub's handlers. A
//! `span` cannot hold a list item, so the caller picks the element (mailo gaps 2): a `span`
//! (the default, inline), a `div`, or an `li` that is itself the list's item. The handlers are
//! the same on each, so the hover-intent machine sees one kind of target whatever it is drawn
//! as. A wrapper with no box of its own (`display:contents`) is not offered: the card is placed
//! against the target's measured rect, and such an element has none.

use super::{Anchors, kind_slug, use_anchors};
use crate::geometry::MountedRef;
use crate::motion::hover_intent::HoverEvent;
use crate::overlay::hover_hub::{HoverHub, HoverKey, HoverKind, use_hover_hub};
use crate::overlay::stack::LayerStack;
use dioxus::prelude::*;

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
    hub: HoverHub,
    anchors: Anchors,
    stack: Option<Signal<LayerStack>>,
    element: Signal<Option<MountedRef>>,
    key: HoverKey,
    kind: HoverKind,
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
        // No card while a peek, the palette or a menu is open (`S:1790`).
        if self.stack.is_some_and(|stack| stack.peek().top().is_some()) {
            self.hub.feed(HoverEvent::OverSuppressed);
            return;
        }
        if let Some(mounted) = self.element.peek().clone() {
            self.anchors.record(self.key.clone(), mounted);
        }
        self.hub
            .feed(HoverEvent::Over((self.key.clone(), self.kind)));
    }

    /// The pointer left the target.
    fn out(&self) {
        self.hub.feed(HoverEvent::Out);
    }

    /// A click removes the card at once, not warm (`S:1809`).
    fn down(&self) {
        self.hub.feed(HoverEvent::ClickInList);
    }
}

/// Wraps whatever a card hooks, drawn as `as_`: feeds the hover hub.
///
/// The component doc names the first prop `key`; dioxus reserves `key` for list identity and
/// rejects a prop of that name, so it is `hover_key` (FINDINGS.md).
#[component]
pub fn HoverTarget(
    hover_key: HoverKey,
    kind: HoverKind,
    #[props(default)] as_: TargetElement,
    children: Element,
) -> Element {
    let hooks = Hooks {
        hub: use_hover_hub(),
        anchors: use_anchors(),
        stack: try_use_context::<Signal<LayerStack>>(),
        element: use_signal(|| None::<MountedRef>),
        key: hover_key.clone(),
        kind,
    };
    let (mount, over, out, down) = (hooks.clone(), hooks.clone(), hooks.clone(), hooks);
    let slug = kind_slug(kind);
    match as_ {
        TargetElement::Span => rsx! {
            span {
                class: "ds-hover-target",
                "data-hover-key": "{hover_key.0}",
                "data-kind": slug,
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
