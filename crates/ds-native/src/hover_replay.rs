//! Carrying out `crate::hover_sync` against a document after a resolve (sill Q170): put Blitz's
//! hover back where it was without events, then replay the last pointer event as a move, so
//! Blitz's own driver dispatches `pointerout`/`pointerleave` for what the pointer left and
//! `pointerover`/`pointerenter` for what came under it, with Dioxus delivery, exactly as for a
//! real move. The replayed `pointermove` is harmless: a browser sends the same "fake" move when
//! layout slides content under a resting mouse.

use crate::hover_sync::{HoverShift, HoverSync, Probe, Rest, decide, probe_points};
use blitz_dom::{BaseDocument, Document as _, NodeId};
use blitz_traits::events::{BlitzPointerEvent, MouseEventButton, UiEvent};
use dioxus_native_dom::DioxusDocument;
use ds::{Point, Px, Rect, Size};

/// The last pointer event the document was sent: where the pointer rests, with the buttons and
/// modifiers it last reported, so the replay is the move a real one there would be.
#[derive(Debug, Clone)]
pub(crate) struct RestingPointer(BlitzPointerEvent);

impl RestingPointer {
    /// The pointer as `event` left it.
    pub(crate) fn from_event(event: &UiEvent) -> Option<Self> {
        match event {
            UiEvent::PointerMove(pointer)
            | UiEvent::PointerDown(pointer)
            | UiEvent::PointerUp(pointer) => Some(RestingPointer(pointer.clone())),
            _ => None,
        }
    }

    /// A move to where the pointer rests (a move names no button of its own).
    fn replay(&self) -> UiEvent {
        UiEvent::PointerMove(BlitzPointerEvent {
            button: MouseEventButton::Main,
            ..self.0.clone()
        })
    }
}

/// Whether a sync replayed a move: then its handlers may have queued renders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Synced {
    /// No: the hover had not moved by itself.
    Still,
    /// Yes.
    Replayed,
}

/// After a resolve that left `doc` hovering something other than `before`, bring Blitz's hover
/// back to `before` and replay `resting` so the change is dispatched.
pub(crate) fn sync(
    doc: &mut DioxusDocument,
    before: Option<NodeId>,
    resting: Option<&RestingPointer>,
) -> Synced {
    let rest = resting.map_or(Rest::Unknown, |_| Rest::Known);
    let decision = {
        let inner = doc.inner.borrow();
        let shift = HoverShift {
            before,
            after: inner.get_hover_node_id(),
        };
        decide(shift, rest, probes(&inner, before))
    };
    let Some(resting) = resting else {
        return Synced::Still;
    };
    match decision {
        HoverSync::Unchanged => return Synced::Still,
        HoverSync::Restore(at) => {
            doc.inner.borrow_mut().set_hover_to(at.x.0, at.y.0);
        }
        HoverSync::Clear => {
            doc.inner.borrow_mut().clear_hover();
        }
    }
    doc.handle_ui_event(resting.replay());
    Synced::Replayed
}

/// Points inside `element`'s current box, in the page coordinates `hit` takes, each with what
/// it hits now. A hit is mapped with `nearest_non_anonymous_ancestor`, the mapping Blitz applies
/// before it stores a hovered node, so a probe matches exactly when hovering it restores `before`.
fn probes(doc: &BaseDocument, element: Option<NodeId>) -> impl Iterator<Item = Probe> + '_ {
    let scroll = doc.viewport_scroll();
    element
        .and_then(|id| doc.get_client_bounding_rect(id))
        .map(|found| {
            probe_points(Rect {
                origin: Point {
                    x: Px((found.x + scroll.x) as f32),
                    y: Px((found.y + scroll.y) as f32),
                },
                size: Size {
                    width: Px(found.width as f32),
                    height: Px(found.height as f32),
                },
            })
        })
        .into_iter()
        .flatten()
        .map(move |at| Probe {
            at,
            element: doc
                .hit(at.x.0, at.y.0)
                .and_then(|hit| doc.nearest_non_anonymous_ancestor(hit.node_id)),
        })
}
