//! HoverCard: a preview that opens after the pointer rests, never marks read, never fetches
//! (design/04-COMPONENTS.md section 22, design/06-INTERACTIONS.md section 3).
//!
//! A `HoverTarget` feeds the `HoverHub` (the Card profile: 500 ms to open, 0 when warm; 150 ms to
//! close; warm for 400 ms after) and records its rect when the pointer comes over it. The
//! consumer renders a `HoverCard` for `hub.open().or(hub.leaving())`, keyed by the hover key so a
//! replacement plays its own entrance; the card places itself against that target by kind, with
//! no flip (section 3 "Positioning"), and fades in and out over `--t-quick` (design/30 section
//! 1.3: no spring). Its content is a list of [`HoverCardPart`]s (the section's blocks as data),
//! then any children.

pub(crate) mod intent;
pub(crate) mod parts;
pub(crate) mod target;

use crate::components::overlays::flow::Flow;
use crate::components::overlays::popover::{Float, Stacking, position_style, use_float};
use crate::host::measure::MountedRef;
use crate::host::measure::client_rect;
use crate::root::common::Common;
use crate::stack::hover_hub::{HoverKey, HoverKind, use_hover_hub};
use dioxus::core::provide_root_context;
use dioxus::prelude::*;
use ds_core::geometry::{
    placement::{Align, Placement, Side},
    units::{Point, Px, Rect},
};
use ds_core::time::{FRAME_SLACK, clock::sleep};
use ds_motion::anim::Anim;
use ds_motion::entrance::use_entrance;
use ds_motion::hover_intent::HoverEvent;
use ds_style::tokens::layer::ZLayer;
use std::collections::BTreeMap;
use {intent::HoverAnchor, parts::HoverCardPart};

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
        // On the root, not the caller's scope: a target that re-renders must not lose its answer.
        ds_style::task::spawn_in(ScopeId::ROOT, async move {
            // Layout may not have reached the element yet: ask again a few frames running.
            for _ in 0..60 {
                sleep(FRAME_SLACK).await;
                if let Some(rect) = client_rect(&element.0).await {
                    let mut book = self.0;
                    book.with_mut(|book| book.insert(key, rect));
                    return;
                }
            }
        });
    }
}

/// The `data-kind` word.
pub(super) fn kind_slug(kind: HoverKind) -> &'static str {
    match kind {
        HoverKind::Thread => "thread",
        HoverKind::Sender => "sender",
        HoverKind::Account => "account",
        HoverKind::Side => "side",
    }
}

/// What a hover surface stands beside: a hover card of some kind, a tooltip, or a dock label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Standing {
    /// A hover card of this content kind.
    Card(HoverKind),
    /// A tooltip: 6 below the target, centred, flipping above when it would overflow.
    Tip,
    /// A dock label: 6 above the target, centred, flipping below when it would overflow.
    Label,
}

/// Where a surface standing as `standing` goes against its target (`S:1715-1726`): a thread card
/// 10 right of the row and 4 above its top; account and side cards 10 right and 6 above; a
/// sender card 6 below the target's left edge. Never flipped: a card that would overflow slides
/// along the edge.
fn card_placement(standing: Standing, target: Rect) -> (Rect, Placement, Px) {
    let kind = match standing {
        Standing::Card(kind) => kind,
        Standing::Tip => {
            return (target, Placement::new(Side::Bottom, Align::Center), Px(6.0));
        }
        Standing::Label => return (target, Placement::new(Side::Top, Align::Center), Px(6.0)),
    };
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
        HoverKind::Sender => (
            target,
            Placement::new(Side::Bottom, Align::Start).no_flip(),
            Px(6.0),
        ),
    }
}

/// A hover-driven surface's floating registration, its `left`/`top`, and its `data-presence`: the
/// surface for the hub's open key (or the one fading out), placed as `standing` against that
/// key's target.
///
/// `own` files the surface under a key of its own instead of the hub's open one: a caller-driven
/// tooltip or label, which the hub is not showing.
pub(crate) fn use_card(
    standing: Standing,
    own: Option<&HoverKey>,
) -> (Float, String, &'static str) {
    let hub = use_hover_hub();
    let float = use_float(ZLayer::Card, Stacking::Passive);
    let anchors = use_anchors();
    let entrance = use_entrance(Anim::PaletteFade);
    let open = hub.open();
    let leaving = hub.leaving();
    let presence = match (&open, &leaving, own) {
        (None, Some(_), None) => "leaving",
        _ => entrance.slug(),
    };
    let target = match own {
        Some(key) => anchors.of(key),
        None => open.or(leaving).and_then(|(key, _)| anchors.of(&key)),
    }
    .map(|rect| card_placement(standing, rect));
    // A hint that stands beside its own target is not drawn until it is placed: the target's rect
    // and its own size are both measured, so it never paints at the overlay's corner first.
    let unplaced = (own.is_some() || !matches!(standing, Standing::Card(_)))
        && (target.is_none() || float.surface().rect().is_none());
    let at = target.map_or(Point::default(), |(rect, want, gap)| {
        float.origin(Some(rect), want, gap)
    });
    let style = if unplaced {
        format!("{};visibility:hidden", position_style(at))
    } else {
        position_style(at)
    };
    (float, style, presence)
}

/// The card, rendered by the consumer for the hub's open key: `parts` in order, then
/// `children` for anything the parts do not draw.
///
/// `flow` is where it is drawn: [`Flow::Floating`] (the default) in the overlay, placed against
/// its key's anchor; [`Flow::Inline`] where the caller renders it, static in the caller's
/// container, with the same markup, entrance and card hooks (a card whose key has
/// no layout to place against, in a test or a server render, is asserted where it stands).
///
/// `common` puts the consumer's `id`, `data-*` and classes on the card, and its `aria_label` names
/// it.
#[component]
pub fn HoverCard(
    kind: HoverKind,
    #[props(default)] parts: Vec<HoverCardPart>,
    #[props(default)] flow: Flow,
    #[props(default)] common: Common,
    children: Element,
) -> Element {
    let hub = use_hover_hub();
    let (float, style, presence) = use_card(Standing::Card(kind), None);
    let class = common.class("ds-popover ds-hovercard");
    let data = common.data_attributes();
    let aria_label = common.aria_label.clone();
    let probe = float.surface();
    let style = match flow {
        Flow::Floating => Some(style),
        Flow::Inline => None,
    };
    let card = rsx! {
        div {
            id: common.id.clone(),
            class,
            "aria-label": aria_label,
            "data-elevation": "pop",
            "data-layer": "card",
            "data-kind": kind_slug(kind),
            "data-flow": flow.attr(),
            "data-presence": presence,
            style,
            onmounted: move |event| {
                common.mounted(event.clone());
                probe.on_mounted(event);
            },
            onmouseenter: move |_| hub.feed(HoverEvent::EnterCard),
            onmouseleave: move |_| hub.feed(HoverEvent::LeaveCard),
            ..data,
            div { class: "ds-hovercard-body",
                for block in parts {
                    {parts::part(block)}
                }
                {children}
            }
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
    use super::{Standing, card_placement};
    use crate::stack::hover_hub::HoverKind;
    use ds_core::geometry::{
        placement::place,
        units::{Point, Px, Rect, Size},
    };

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
        // (standing, target, want x, want y), section 3 "Positioning".
        #[rustfmt::skip]
        let cases = [
            (Standing::Card(HoverKind::Thread), rect(260.0, 100.0, 520.0, 64.0), 790.0, 96.0),
            (Standing::Card(HoverKind::Side), rect(12.0, 300.0, 180.0, 30.0), 202.0, 294.0),
            (Standing::Card(HoverKind::Account), rect(12.0, 20.0, 40.0, 40.0), 62.0, 14.0),
            (Standing::Card(HoverKind::Sender), rect(400.0, 200.0, 90.0, 18.0), 400.0, 224.0),
            // No flip: a sender card at the bottom slides up inside the 8 px margin.
            (Standing::Card(HoverKind::Sender), rect(400.0, 760.0, 90.0, 18.0), 400.0, 652.0),
            // A thread card at the right edge slides left rather than flipping.
            (Standing::Card(HoverKind::Thread), rect(700.0, 100.0, 450.0, 64.0), 892.0, 96.0),
            // A tooltip centres below its target; a dock label centres above it.
            (Standing::Tip, rect(400.0, 200.0, 90.0, 18.0), 295.0, 224.0),
            (Standing::Label, rect(400.0, 400.0, 60.0, 60.0), 280.0, 254.0),
        ];
        for (standing, target, x, y) in cases {
            let (anchor, want, gap) = card_placement(standing, target);
            let at = place(anchor, card, window, want, gap).origin;
            assert_eq!((at.x, at.y), (Px(x), Px(y)), "{standing:?} at {target:?}");
        }
    }
}
