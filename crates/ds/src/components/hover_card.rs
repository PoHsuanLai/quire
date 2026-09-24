//! HoverCard: a preview that opens after the pointer rests, never marks read, never fetches
//! (design/04-COMPONENTS.md section 22, design/06-INTERACTIONS.md section 3).
//!
//! A `HoverTarget` feeds the `HoverHub` (450 ms to open, 0 when warm; 150 ms to close; warm for
//! 400 ms after) and records its rect when the pointer comes over it. The consumer renders a
//! `HoverCard` for `hub.open().or(hub.leaving())`, keyed by the hover key so a replacement
//! plays its own `hc-in`; the card places itself against that target by kind, with no flip
//! (section 3 "Positioning"), and plays `hc-out` while the hub reports it leaving. Its content
//! is a list of [`HoverCardPart`]s (the section's blocks as data), then any children.

mod intent;
mod parts;
mod target;

pub use intent::{HoverAnchor, HoverDriver, use_hover_intent};
pub use parts::{FlagTone, HoverCardPart, HoverMessage, HoverStat, KeyHint};
pub use target::{HoverTarget, TargetElement};

use crate::components::flow::Flow;
use crate::components::popover::{Float, Stacking, position_style, use_entrance, use_float};
use crate::geometry::measure::client_rect;
use crate::geometry::{Align, MountedRef, Placement, Point, Px, Rect, Side};
use crate::motion::anim::Anim;
use crate::motion::hover_intent::HoverEvent;
use crate::overlay::hover_hub::{HoverKey, HoverKind, use_hover_hub};
use crate::time::{FRAME_SLACK, sleep};
use crate::tokens::ZLayer;
use dioxus::core::provide_root_context;
use dioxus::prelude::*;
use std::collections::BTreeMap;

/// Where each hover target was when the pointer last came over it, shared by every target
/// and card under the root.
#[derive(Clone, Copy)]
pub(crate) struct Anchors(Signal<BTreeMap<HoverKey, Rect>>);

/// The shared anchor book, created at the root on first use.
pub(crate) fn use_anchors() -> Anchors {
    use_hook(|| {
        try_consume_context::<Anchors>().unwrap_or_else(|| {
            provide_root_context(Anchors(Signal::new_in_scope(
                BTreeMap::new(),
                ScopeId::ROOT,
            )))
        })
    })
}

impl Anchors {
    /// The target `key`'s last rect.
    pub(crate) fn of(&self, key: &HoverKey) -> Option<Rect> {
        self.0.read().get(key).copied()
    }

    /// File `anchor` under `key`: a rect at once, an element once measured, and nothing (the
    /// key's old rect dropped) for an unplaced anchor.
    pub(crate) fn file(self, key: HoverKey, anchor: HoverAnchor) {
        let mut book = self.0;
        match anchor {
            HoverAnchor::Rect(rect) => {
                book.with_mut(|book| book.insert(key, rect));
            }
            HoverAnchor::Element(element) => self.record(key, element),
            HoverAnchor::Unplaced => {
                if book.peek().contains_key(&key) {
                    book.with_mut(|book| book.remove(&key));
                }
            }
        }
    }

    /// Read `element`'s rect after layout and file it under `key`.
    fn record(self, key: HoverKey, element: MountedRef) {
        spawn(async move {
            sleep(FRAME_SLACK).await;
            if let Some(rect) = client_rect(&element.0).await {
                let mut book = self.0;
                book.with_mut(|book| book.insert(key, rect));
            }
        });
    }
}

/// The `data-kind` word.
fn kind_slug(kind: HoverKind) -> &'static str {
    match kind {
        HoverKind::Thread => "thread",
        HoverKind::Sender => "sender",
        HoverKind::Account => "account",
        HoverKind::Side => "side",
        HoverKind::Tip => "tip",
    }
}

/// Where a card of `kind` goes against its target (`S:1715-1726`): a thread card 10 right of
/// the row and 4 above its top; account and side cards 10 right and 6 above; the rest (a
/// sender card, a time tip) 6 below the target's left edge. Never flipped: a card that would overflow slides along the edge.
fn card_placement(kind: HoverKind, target: Rect) -> (Rect, Placement, Px) {
    let raised = |by: f32| Rect {
        origin: Point {
            y: target.origin.y - Px(by),
            ..target.origin
        },
        ..target
    };
    match kind {
        HoverKind::Thread => (
            raised(4.0),
            Placement::new(Side::Right, Align::Start).no_flip(),
            Px(10.0),
        ),
        HoverKind::Account | HoverKind::Side => (
            raised(6.0),
            Placement::new(Side::Right, Align::Start).no_flip(),
            Px(10.0),
        ),
        HoverKind::Sender | HoverKind::Tip => (
            target,
            Placement::new(Side::Bottom, Align::Start).no_flip(),
            Px(6.0),
        ),
    }
}

/// A hover-driven card's floating registration, its `left`/`top`, and its `data-presence`: the
/// card for the hub's open key (or the one playing `hc-out`), placed as `kind` against that
/// key's target.
pub(crate) fn use_card(kind: HoverKind) -> (Float, String, &'static str) {
    let hub = use_hover_hub();
    let float = use_float(ZLayer::Card, Stacking::Passive);
    let anchors = use_anchors();
    let entrance = use_entrance(Anim::HcIn);
    let open = hub.open();
    let leaving = hub.leaving();
    let presence = match (&open, &leaving) {
        (None, Some(_)) => "leaving",
        _ => entrance.slug(),
    };
    let target = open
        .or(leaving)
        .and_then(|(key, _)| anchors.of(&key))
        .map(|rect| card_placement(kind, rect));
    let at = target.map_or(Point::default(), |(rect, want, gap)| {
        float.origin(Some(rect), want, gap)
    });
    (float, position_style(at), presence)
}

/// The card, rendered by the consumer for the hub's open key: `parts` in order, then
/// `children` for anything the parts do not draw.
///
/// `flow` is where it is drawn: [`Flow::Floating`] (the default) in the overlay, placed against
/// its key's anchor; [`Flow::Inline`] where the caller renders it, static in the caller's
/// container, with the same markup, entrance and card hooks (mailo gaps 4: a card whose key has
/// no layout to place against, in a test or a server render, is asserted where it stands).
#[component]
pub fn HoverCard(
    kind: HoverKind,
    #[props(default)] parts: Vec<HoverCardPart>,
    #[props(default)] flow: Flow,
    children: Element,
) -> Element {
    let hub = use_hover_hub();
    let (float, style, presence) = use_card(kind);
    let probe = float.surface();
    let style = match flow {
        Flow::Floating => Some(style),
        Flow::Inline => None,
    };
    let card = rsx! {
        div {
            class: "ds-popover ds-hovercard",
            "data-elevation": "pop",
            "data-layer": "card",
            "data-kind": kind_slug(kind),
            "data-flow": flow.attr(),
            "data-presence": presence,
            style,
            onmounted: move |event| probe.on_mounted(event),
            onmouseenter: move |_| hub.feed(HoverEvent::EnterCard),
            onmouseleave: move |_| hub.feed(HoverEvent::LeaveCard),
            for block in parts {
                {parts::part(block)}
            }
            {children}
        }
    };
    match flow {
        Flow::Floating => {
            float.show(card, EventHandler::new(|()| {}));
            rsx! {}
        }
        Flow::Inline => card,
    }
}

#[cfg(test)]
mod tests {
    use super::card_placement;
    use crate::geometry::{Point, Px, Rect, Size, place};
    use crate::overlay::hover_hub::HoverKind;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect {
            origin: Point { x: Px(x), y: Px(y) },
            size: Size {
                width: Px(w),
                height: Px(h),
            },
        }
    }

    #[test]
    fn each_kind_sits_where_the_prototype_put_it() {
        let window = rect(0.0, 0.0, 1200.0, 800.0);
        let card = Size {
            width: Px(300.0),
            height: Px(140.0),
        };
        // (kind, target, want x, want y), section 3 "Positioning".
        #[rustfmt::skip]
        let cases = [
            (HoverKind::Thread, rect(260.0, 100.0, 520.0, 64.0), 790.0, 96.0),
            (HoverKind::Side, rect(12.0, 300.0, 180.0, 30.0), 202.0, 294.0),
            (HoverKind::Account, rect(12.0, 20.0, 40.0, 40.0), 62.0, 14.0),
            (HoverKind::Sender, rect(400.0, 200.0, 90.0, 18.0), 400.0, 224.0),
            // No flip: a sender card at the bottom slides up inside the 8 px margin.
            (HoverKind::Sender, rect(400.0, 760.0, 90.0, 18.0), 400.0, 652.0),
            // A thread card at the right edge slides left rather than flipping.
            (HoverKind::Thread, rect(700.0, 100.0, 450.0, 64.0), 892.0, 96.0),
        ];
        for (kind, target, x, y) in cases {
            let (anchor, want, gap) = card_placement(kind, target);
            let at = place(anchor, card, window, want, gap).origin;
            assert_eq!((at.x, at.y), (Px(x), Px(y)), "{kind:?} at {target:?}");
        }
    }
}
