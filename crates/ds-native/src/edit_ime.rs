//! Routing the IME to the edit surface that has the keyboard (FINDINGS "Edit surface", route
//! b). dioxus-native-dom drops `DomEventData::Ime`, so no Dioxus handler hears it; ds-native
//! sees it first instead (the window's `use_window_event` runs before the document gets the
//! event; the harness owns its document) and hands it to the surface registered at the focused
//! node or its nearest registered ancestor.

use blitz_dom::{BaseDocument, NodeId};
use dioxus::prelude::EventHandler;
use dioxus_native::winit::event::Ime;
use ds::{CapturedPointer, ImeEvent, ImeListener, PointerPhase};
use std::cell::RefCell;
use std::rc::Rc;

/// The edit surfaces of one document listening for IME events, provided as root context.
#[derive(Clone, Default)]
pub(crate) struct EditListeners(Rc<RefCell<Registry>>);

#[derive(Default)]
struct Registry {
    next: u64,
    entries: Vec<Entry>,
    /// The surface a press captured the pointer for, until the release.
    captured: Option<EventHandler<CapturedPointer>>,
}

struct Entry {
    id: ImeListener,
    node: NodeId,
    sink: EventHandler<ImeEvent>,
}

impl EditListeners {
    /// Deliver IME events to `sink` while `node` or a node inside it has the focus.
    pub(crate) fn add(&self, node: NodeId, sink: EventHandler<ImeEvent>) -> ImeListener {
        let mut registry = self.0.borrow_mut();
        registry.next += 1;
        let id = ImeListener(registry.next);
        registry.entries.push(Entry { id, node, sink });
        id
    }

    /// Stop delivering to `id`.
    pub(crate) fn remove(&self, id: ImeListener) {
        self.0.borrow_mut().entries.retain(|entry| entry.id != id);
    }

    /// Route the pointer to `sink` until the release.
    pub(crate) fn capture(&self, sink: EventHandler<CapturedPointer>) {
        self.0.borrow_mut().captured = Some(sink);
    }

    /// The captured surface's sink for `phase`, released with it at the release. `None` when
    /// no surface holds the pointer.
    pub(crate) fn captured(&self, phase: PointerPhase) -> Option<EventHandler<CapturedPointer>> {
        let mut registry = self.0.borrow_mut();
        match phase {
            PointerPhase::Release => registry.captured.take(),
            PointerPhase::Press | PointerPhase::Drag => registry.captured,
        }
    }

    /// The listener for `doc`'s focused node: the one registered nearest above it.
    pub(crate) fn target(&self, doc: &BaseDocument) -> Option<EventHandler<ImeEvent>> {
        let registry = self.0.borrow();
        let mut at = doc.get_focussed_node_id();
        while let Some(id) = at {
            if let Some(entry) = registry.entries.iter().rev().find(|entry| entry.node == id) {
                return Some(entry.sink);
            }
            at = doc.get_node(id).and_then(|node| node.parent);
        }
        None
    }
}

/// The surface's vocabulary for a winit IME event; `None` for what a surface does not take
/// (surrounding-text deletion: the surface never offers surrounding text).
pub(crate) fn ime_of(event: &Ime) -> Option<ImeEvent> {
    match event {
        Ime::Enabled => Some(ImeEvent::Enabled),
        Ime::Preedit(text, cursor) => Some(ImeEvent::Preedit {
            text: text.clone(),
            cursor: *cursor,
        }),
        Ime::Commit(text) => Some(ImeEvent::Commit(text.clone())),
        Ime::Disabled => Some(ImeEvent::Disabled),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::ime_of;
    use dioxus_native::winit::event::Ime;
    use ds::ImeEvent;

    #[test]
    fn winit_ime_events_become_the_surfaces() {
        let cases = [
            (Ime::Enabled, Some(ImeEvent::Enabled)),
            (
                Ime::Preedit("ㄓ".to_owned(), Some((3, 3))),
                Some(ImeEvent::Preedit {
                    text: "ㄓ".to_owned(),
                    cursor: Some((3, 3)),
                }),
            ),
            (
                Ime::Commit("注".to_owned()),
                Some(ImeEvent::Commit("注".to_owned())),
            ),
            (Ime::Disabled, Some(ImeEvent::Disabled)),
            (
                Ime::DeleteSurrounding {
                    before_bytes: 1,
                    after_bytes: 0,
                },
                None,
            ),
        ];
        for (event, expected) in cases {
            assert_eq!(ime_of(&event), expected, "{event:?}");
        }
    }
}
