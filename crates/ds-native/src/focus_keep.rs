//! The keyboard after the element that had it leaves the document (mailo gaps 7).
//!
//! Blitz resets the focus to nowhere when the focused node is removed
//! (`clear_interaction_state_for_removed_node`), so a menu that closed on Escape, or a field
//! whose own Enter removed it, left every later key going to the document's root instead of
//! the app's `.app[tabindex]`. Under `FocusFallback::Ancestor` the host looks at the focus after
//! each settled frame (the harness) or window event (the window, before the document hears the
//! event), and when the element it last saw focused is gone and the focus is nowhere, focuses
//! the nearest focusable ancestor that element had, remembered while it was there (Blitz severs
//! a removed node from its parent). A surface a component registered through `ds::HostHandBack`
//! (a floating menu's panel, whose own ancestors are the overlay layer) gives it to the element
//! it named first (the menu's anchor). After those, the element focused before the removed one
//! and then the element under the pointer when the keyboard moved to it (the button whose press
//! opened the menu, when the menu's own focus overtook the press's): the opener, in both cases.
//!
//! A focus cleared on purpose (a field's blur, a click on nothing) leaves the element in the
//! document, and the keeper then does nothing, even if the element leaves later.

use crate::click_focus::nowhere;
use crate::focus_chain::{Candidates, Mark, Presence};
use crate::node_ref::{DocRef, NodeRef, Written};
use blitz_dom::{BaseDocument, NodeId};
use dioxus::prelude::MountedData;
use ds::HostHandBack;
use std::cell::RefCell;
use std::rc::Rc;

/// The element last seen with the keyboard, and where the keyboard could go without it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Held {
    node: Mark,
    /// The element itself (if focusable) and its focusable ancestors, nearest first.
    candidates: Candidates,
}

/// What the keeper last saw.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum Seen {
    /// Nothing focused yet, or the keeper just moved the focus.
    #[default]
    Nothing,
    /// An element has the keyboard.
    On(Held),
    /// The focus was cleared while the element stayed: on purpose, so it is left there.
    Released(Held),
}

impl Seen {
    fn held(&self) -> Option<&Held> {
        match self {
            Seen::Nothing => None,
            Seen::On(held) | Seen::Released(held) => Some(held),
        }
    }
}

/// The host's memory of the focus, one per document.
#[derive(Debug, Clone, Default)]
pub(crate) struct FocusKeeper {
    seen: Seen,
    /// The candidates of the element focused before the last one, then those under the pointer
    /// when the focus moved.
    before: Candidates,
    /// Surfaces that give the keyboard back to a named element (`ds::HostHandBack`).
    returns: Vec<HandBack>,
}

/// One registered hand-back: when `surface` is removed with the keyboard, `opener` has it next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HandBack {
    surface: NodeId,
    opener: NodeId,
}

impl FocusKeeper {
    /// Look at `doc`'s focus: remember a newly focused element, and answer the element to focus
    /// when the one that had the keyboard was removed.
    pub(crate) fn look(&mut self, doc: &BaseDocument) -> Option<NodeId> {
        match focused(doc) {
            Some(held) => {
                if self.seen.held().map(|seen| &seen.node) != Some(&held.node) {
                    let seen = self.seen.held().map(|seen| seen.candidates.clone());
                    self.before = seen.unwrap_or_default().then(&under_pointer(doc));
                }
                self.seen = Seen::On(held);
                None
            }
            None => self.lost(doc),
        }
    }

    /// Record that `surface`, removed with the keyboard, gives it to `opener`.
    pub(crate) fn hand_back(&mut self, surface: NodeId, opener: NodeId) {
        self.returns.retain(|back| back.surface != surface);
        self.returns.push(HandBack { surface, opener });
    }

    /// The candidates `surface`'s registered opener offers now, and the registrations of
    /// surfaces no longer in `doc` forgotten.
    fn opener_of(&mut self, doc: &BaseDocument, surface: NodeId) -> Candidates {
        let opener = self
            .returns
            .iter()
            .find(|back| back.surface == surface)
            .map(|back| back.opener)
            .filter(|&opener| in_document(doc, opener));
        self.returns.retain(|back| in_document(doc, back.surface));
        opener.map_or_else(Candidates::default, |opener| Candidates::from(doc, opener))
    }

    /// The focus is nowhere: removed, or cleared on purpose.
    fn lost(&mut self, doc: &BaseDocument) -> Option<NodeId> {
        let Seen::On(held) = self.seen.clone() else {
            return None;
        };
        match held.node.presence(doc) {
            Presence::Here => {
                self.seen = Seen::Released(held);
                None
            }
            Presence::Gone => {
                let next = self
                    .opener_of(doc, held.node.node())
                    .then(&held.candidates)
                    .then(&self.before)
                    .first_live(doc);
                self.seen = Seen::Nothing;
                next
            }
        }
    }
}

/// The focusable elements from the one under the pointer up.
fn under_pointer(doc: &BaseDocument) -> Candidates {
    doc.get_hover_node_id()
        .map_or_else(Candidates::default, |node| Candidates::from(doc, node))
}

/// Whether `node` is in `doc`'s tree.
fn in_document(doc: &BaseDocument, node: NodeId) -> bool {
    doc.get_node(node)
        .is_some_and(|found| found.flags.is_in_document())
}

/// The element with the keyboard, unless it is nowhere.
fn focused(doc: &BaseDocument) -> Option<Held> {
    if nowhere(doc) {
        return None;
    }
    let id = doc.get_focussed_node_id()?;
    let node = Mark::of(doc.get_node(id)?)?;
    Some(Held {
        node,
        candidates: Candidates::from(doc, id),
    })
}

/// Look at the document behind `doc` and focus what the keeper answers. A document the renderer
/// holds is looked at next time.
pub(crate) fn keep(keeper: &mut FocusKeeper, doc: &DocRef) -> Kept {
    let before = keeper.clone();
    let Some(Some(next)) = doc.read(|read| keeper.look(read)) else {
        return Kept::Still;
    };
    match doc.write(|write| {
        write.set_focus_to(next);
        write.shell_provider.request_redraw();
    }) {
        Written::Done => Kept::Moved,
        Written::Busy => {
            // Look again next time, from what was known before this look.
            *keeper = before;
            Kept::Still
        }
    }
}

/// Whether the keeper moved the focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kept {
    /// It focused an ancestor of a removed element.
    Moved,
    /// Nothing to do.
    Still,
}

/// The `ds::HostHandBack` that records into `keeper`. A registration made while the keeper is
/// busy looking is dropped; the keeper's own fallbacks still apply.
pub(crate) fn hand_back_seam(keeper: Rc<RefCell<FocusKeeper>>) -> HostHandBack {
    HostHandBack(Rc::new(
        move |surface: &MountedData, opener: &MountedData| {
            if let (Some(surface), Some(opener), Ok(mut keeper)) = (
                NodeRef::of(surface),
                NodeRef::of(opener),
                keeper.try_borrow_mut(),
            ) {
                keeper.hand_back(surface.node, opener.node);
            }
        },
    ))
}
