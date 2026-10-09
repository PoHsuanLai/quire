//! One node of a Blitz document, however the app's handle on it was made: the `NodeHandle` a
//! component's `onmounted` gives, or a [`FoundNode`] ds-blitz makes for an element found by
//! selector (`ds::focus::selector::focus_by_selector`), which dioxus-native-dom has no public way
//! to make a `NodeHandle` for. The focus writes (`crate::focus`) take either.

use blitz_dom::{BaseDocument, NodeId};
use dioxus::html::geometry::{
    PixelsRect,
    euclid::{Point2D, Size2D},
};
use dioxus::html::{MountedError, MountedResult, RenderedElementBacking};
use dioxus::prelude::MountedData;
use dioxus_native_dom::NodeHandle;
use ds::host::found::SameNode;
use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

/// The document a node lives in.
#[derive(Clone)]
pub enum DocRef {
    /// Reached through a mounted element of the window's document (the host's hidden element).
    Handle(NodeHandle),
    /// Held directly: the harness's own document.
    Cell(Rc<RefCell<BaseDocument>>),
}

impl std::fmt::Debug for DocRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocRef").finish_non_exhaustive()
    }
}

impl DocRef {
    /// Read the document, or `None` while it is borrowed mutably.
    pub(crate) fn read<T>(&self, read: impl FnOnce(&BaseDocument) -> T) -> Option<T> {
        match self {
            DocRef::Handle(handle) => handle.try_doc().map(|doc| read(&doc)),
            DocRef::Cell(cell) => cell.try_borrow().ok().map(|doc| read(&doc)),
        }
    }

    /// Write the document, or answer [`Written::Busy`] while it is borrowed.
    pub(crate) fn write(&self, write: impl FnOnce(&mut BaseDocument)) -> Written {
        match self {
            DocRef::Handle(handle) => {
                if handle.try_doc().is_none() {
                    return Written::Busy;
                }
                // Free a moment ago, on this thread: only a shared borrow held further up the
                // stack could refuse the write now, and that is busy too.
                match catch_unwind(AssertUnwindSafe(|| write(&mut handle.doc_mut()))) {
                    Ok(()) => Written::Done,
                    Err(_) => Written::Busy,
                }
            }
            DocRef::Cell(cell) => match cell.try_borrow_mut() {
                Ok(mut doc) => {
                    write(&mut doc);
                    Written::Done
                }
                Err(_) => Written::Busy,
            },
        }
    }
}

/// Whether a write to the document went in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Written {
    /// It did.
    Done,
    /// The document is borrowed (the renderer holds it): try again next frame.
    Busy,
}

/// A node and its document.
#[derive(Clone)]
pub(crate) struct NodeRef {
    pub(crate) doc: DocRef,
    pub(crate) node: NodeId,
}

impl NodeRef {
    /// The node behind a mounted handle, if it is a Blitz node.
    pub(crate) fn of(element: &MountedData) -> Option<NodeRef> {
        if let Some(handle) = element.downcast::<NodeHandle>() {
            return Some(NodeRef {
                doc: DocRef::Handle(handle.clone()),
                node: handle.node_id(),
            });
        }
        element
            .downcast::<FoundNode>()
            .map(|FoundNode(node)| node.clone())
    }

    /// Read the node's document, or `None` while it is borrowed mutably.
    pub(crate) fn read<T>(&self, read: impl FnOnce(&BaseDocument) -> T) -> Option<T> {
        self.doc.read(read)
    }

    /// Write the node's document, or answer [`Written::Busy`] while it is borrowed.
    pub(crate) fn write(&self, write: impl FnOnce(&mut BaseDocument)) -> Written {
        self.doc.write(write)
    }
}

/// An element found by selector, as a mounted handle the focus writes and the host's rect read
/// accept, and the element's `get_client_rect`. It answers no other mounted-element call
/// (scrolling): those need a component's own `onmounted`.
#[derive(Clone)]
pub(crate) struct FoundNode(pub(crate) NodeRef);

impl RenderedElementBacking for FoundNode {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_client_rect(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = MountedResult<PixelsRect>>>> {
        let read = self
            .0
            .read(|doc| doc.get_client_bounding_rect(self.0.node))
            .flatten();
        Box::pin(async move {
            read.map(|found| {
                PixelsRect::new(
                    Point2D::new(found.x, found.y),
                    Size2D::new(found.width, found.height),
                )
            })
            .ok_or(MountedError::NotSupported)
        })
    }
}

/// Whether two handles name the same node (`ds::host::parts::GeometryHost::same`).
pub(crate) fn same(a: &MountedData, b: &MountedData) -> SameNode {
    match (NodeRef::of(a), NodeRef::of(b)) {
        (Some(a), Some(b)) if a.node == b.node => SameNode::Same,
        _ => SameNode::Different,
    }
}
