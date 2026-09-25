//! Where the keyboard may land when the element meant to have it is gone (mailo gaps 7): a run
//! of focusable elements, nearest first, remembered while they were in the document.
//!
//! Remembered, not read at the moment of focusing, because Blitz severs a removed node from its
//! parent: by the time a click's restore or the removed-focus fallback runs, the element it
//! meant is no longer anyone's child and its ancestors cannot be walked. Each candidate is
//! checked for life at the moment of focusing (still in the document, still the same kind of
//! element, still focusable), so a candidate the click itself removed is skipped for the next.

use crate::node_ref::{DocRef, NodeRef};
use blitz_dom::{BaseDocument, LocalName, Node, NodeId};
use dioxus::html::RenderedElementBacking;

/// One remembered element: its id, and its tag, so an id Blitz reused for a new element of
/// another kind is not taken for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Mark {
    node: NodeId,
    tag: LocalName,
}

/// Whether a remembered element is still the one that was remembered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Presence {
    /// In the document, with the same tag.
    Here,
    /// Removed (or its id now names another kind of element).
    Gone,
}

impl Mark {
    /// The element `node` is, if it is an element.
    pub(crate) fn of(node: &Node) -> Option<Mark> {
        node.element_data().map(|element| Mark {
            node: node.id,
            tag: element.name.local.clone(),
        })
    }

    /// The remembered node's id.
    pub(crate) fn node(&self) -> NodeId {
        self.node
    }

    /// Whether the element is still in `doc`.
    pub(crate) fn presence(&self, doc: &BaseDocument) -> Presence {
        match doc.get_node(self.node) {
            Some(node)
                if node.flags.is_in_document()
                    && node
                        .element_data()
                        .is_some_and(|element| element.name.local == self.tag) =>
            {
                Presence::Here
            }
            _ => Presence::Gone,
        }
    }

    /// Whether the element is still in `doc` and can take the keyboard.
    fn takes_focus(&self, doc: &BaseDocument) -> bool {
        self.presence(doc) == Presence::Here
            && doc.get_node(self.node).is_some_and(Node::is_focussable)
    }
}

/// Focusable elements, nearest first.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Candidates(Vec<Mark>);

impl Candidates {
    /// `start` (if focusable) and each focusable ancestor of it, as `doc` has them now.
    pub(crate) fn from(doc: &BaseDocument, start: NodeId) -> Candidates {
        Candidates(
            std::iter::successors(doc.get_node(start), |node| {
                node.parent.and_then(|id| doc.get_node(id))
            })
            .filter(|node| node.is_focussable())
            .filter_map(Mark::of)
            .collect(),
        )
    }

    /// The first candidate that can take the keyboard in `doc` now.
    pub(crate) fn first_live(&self, doc: &BaseDocument) -> Option<NodeId> {
        self.0
            .iter()
            .find(|mark| mark.takes_focus(doc))
            .map(Mark::node)
    }

    /// These candidates, then `more`.
    pub(crate) fn then(&self, more: &Candidates) -> Candidates {
        Candidates(self.0.iter().chain(more.0.iter()).cloned().collect())
    }
}

/// Candidates as a mounted handle, so `ds::HostClickFocus::restore` receives the whole run the
/// click found rather than one element that may be gone by the time it runs.
#[derive(Clone)]
pub(crate) struct ChainNode {
    pub(crate) doc: DocRef,
    pub(crate) candidates: Candidates,
}

impl RenderedElementBacking for ChainNode {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Where `restore` may put the keyboard: a remembered run, or (for a handle the app holds, a
/// menu's anchor) that element or its nearest focusable ancestor as the document has them when
/// it runs.
pub(crate) enum Target {
    /// A run remembered when it was asked for.
    Chain(ChainNode),
    /// One element, walked from when focusing.
    Element(NodeRef),
}

impl Target {
    /// The target behind a mounted handle, if it is a Blitz one.
    pub(crate) fn of(element: &dioxus::prelude::MountedData) -> Option<Target> {
        if let Some(chain) = element.downcast::<ChainNode>() {
            return Some(Target::Chain(chain.clone()));
        }
        NodeRef::of(element).map(Target::Element)
    }

    /// The document it lives in.
    pub(crate) fn doc(&self) -> &DocRef {
        match self {
            Target::Chain(chain) => &chain.doc,
            Target::Element(node) => &node.doc,
        }
    }

    /// The element to focus in `doc` now, if any candidate is alive and focusable.
    pub(crate) fn pick(&self, doc: &BaseDocument) -> Option<NodeId> {
        match self {
            Target::Chain(chain) => chain.candidates.first_live(doc),
            Target::Element(node) => {
                let alive = doc
                    .get_node(node.node)
                    .is_some_and(|found| found.flags.is_in_document());
                alive
                    .then(|| Candidates::from(doc, node.node).first_live(doc))
                    .flatten()
            }
        }
    }
}
