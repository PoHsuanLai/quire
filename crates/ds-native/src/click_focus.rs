//! A click on nothing focusable, on Blitz (Native focus): the nearest focusable ancestor keeps
//! or takes the keyboard, as in a browser, where Blitz would clear it (`handle_click`'s "nothing
//! matched"). `ds::Ds`'s root hands every click nothing inside took to [`CLICK_FOCUS`], after
//! the app's handlers and before Blitz's default action.
//!
//! The default is never prevented: it also dispatches `dblclick` and the `blur` of a field the
//! click leaves. So when the ancestor already has the focus, it is cleared here without events
//! (Blitz's clear then has nothing to clear, and the ancestor hears no spurious blur); either
//! way the ancestor is focused a frame later, if the click left the focus nowhere.

use crate::node_ref::{FoundNode, NodeRef, Written};
use blitz_dom::{BaseDocument, LocalName, Node, NodeId};
use dioxus::prelude::*;
use ds::{Fallback, Focused, HostClickFocus};
use std::rc::Rc;

/// Where the keyboard goes after a click on nothing focusable, for a window or a harness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FocusFallback {
    /// To the nearest focusable ancestor (a `tabindex`, or a natively focusable element), as
    /// in a browser; nowhere if there is none.
    #[default]
    Ancestor,
    /// Blitz's own: the focus is cleared.
    BlitzDefault,
}

/// ds-native's click-focus seam, provided under [`FocusFallback::Ancestor`].
pub const CLICK_FOCUS: HostClickFocus = HostClickFocus { fallback, restore };

/// What a click at the document's hover node will do to the focus, read through `root`.
fn fallback(root: &MountedData) -> Fallback {
    let Some(root) = NodeRef::of(root) else {
        return Fallback::Renderer;
    };
    let Some(Some(ancestor)) = root.read(ancestor_of_click) else {
        return Fallback::Renderer;
    };
    let target = NodeRef {
        doc: root.doc.clone(),
        node: ancestor,
    };
    // Already there: clear it without events, so Blitz's clear has nothing to blur, and give it
    // back after the click.
    if root.read(|doc| doc.get_focussed_node_id() == Some(ancestor)) == Some(true) {
        let _ = target.write(BaseDocument::clear_focus);
    }
    Fallback::Ancestor(Rc::new(MountedData::new(FoundNode(target))))
}

/// Focus `ancestor` if the focus is nowhere: a handler that moved it during the click wins.
fn restore(ancestor: &MountedData) -> Focused {
    let Some(node) = NodeRef::of(ancestor) else {
        return Focused::Unknown;
    };
    let nowhere = node.read(|doc| {
        let root = doc.try_root_element().map(|root| root.id);
        doc.get_focussed_node_id()
            .is_none_or(|focused| Some(focused) == root)
    });
    match nowhere {
        None => Focused::Busy,
        Some(false) => Focused::Unknown,
        Some(true) => match node.write(|doc| {
            doc.set_focus_to(node.node);
            doc.shell_provider.request_redraw();
        }) {
            Written::Done => Focused::Done,
            Written::Busy => Focused::Busy,
        },
    }
}

/// The focusable ancestor a click at the hover node should leave the focus on, when Blitz would
/// clear it: `None` when Blitz handles the click itself or nothing on the way up is focusable.
pub(crate) fn ancestor_of_click(doc: &BaseDocument) -> Option<NodeId> {
    let target = doc.get_hover_node_id()?;
    let chain: Vec<&Node> = std::iter::successors(doc.get_node(target), |node| {
        node.parent.and_then(|id| doc.get_node(id))
    })
    .collect();
    if chain.iter().any(|node| blitz_takes(node)) {
        return None;
    }
    chain
        .iter()
        .find(|node| node.is_focussable())
        .map(|node| node.id)
}

/// Whether Blitz's `handle_click` acts on this element, and so does not clear the focus: a
/// disabled element, a text field, any `input`, a submit button, a `summary`, a `label`, a link.
fn blitz_takes(node: &Node) -> bool {
    let Some(element) = node.element_data() else {
        return false;
    };
    if element.attr(LocalName::from("disabled")).is_some() || element.text_input_data().is_some() {
        return true;
    }
    match element.name.local.as_ref() {
        "input" | "summary" | "label" => true,
        "button" => element.is_submit_button(),
        "a" => element.attr(LocalName::from("href")).is_some(),
        _ => false,
    }
}
