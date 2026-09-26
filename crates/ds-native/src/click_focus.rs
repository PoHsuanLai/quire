//! A click on nothing focusable, on Blitz (Native focus): the nearest focusable ancestor keeps
//! or takes the keyboard, as in a browser, where Blitz would clear it (`handle_click`'s "nothing
//! matched"). `ds::Ds`'s root hands every click nothing inside took to [`CLICK_FOCUS`], after
//! the app's handlers and before Blitz's default action.
//!
//! The default is never prevented: it also dispatches `dblclick` and the `blur` of a field the
//! click leaves. So when the ancestor already has the focus, it is cleared here without events
//! (Blitz's clear then has nothing to clear, and the ancestor hears no spurious blur); either
//! way the ancestor is focused a frame later, if the click left the focus nowhere.
//!
//! The click's handler may remove the very element found (mailo's "Show images" button removes
//! itself on press), so the fallback remembers every focusable element from the target up
//! (`crate::focus_chain`), and `restore` focuses the first of them still in the document when it
//! runs (mailo gaps 7).
//!
//! A click a quire control keeps to itself never reaches the root, and quire's control hands it
//! over itself (`ds::focus::click::kept_click`): through [`CLICK_FOCUS`] when the click's default
//! still runs, and through [`PRESS_FOCUS`] when the control prevented it. Blitz then leaves the
//! focus where it was, so `PRESS_FOCUS` moves it at once to the nearest focusable element from
//! the pressed one up, unless a field has the keyboard (no `blur` could tell it so).

use crate::focus_chain::{Candidates, ChainNode, Target};
use crate::node_ref::{NodeRef, Written};
use blitz_dom::{BaseDocument, LocalName, Node, NodeId};
use dioxus::prelude::*;
use ds::{Fallback, Focused, HostClickFocus, HostPressFocus};
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

/// ds-native's press-focus seam, provided beside [`CLICK_FOCUS`] under
/// [`FocusFallback::Ancestor`]: a click a quire control kept with its default prevented gives
/// the pressed control the keyboard.
pub const PRESS_FOCUS: HostPressFocus = HostPressFocus(take);

/// Give the element pressed at the document's hover node (or its nearest focusable ancestor)
/// the keyboard now, read and written through `root`.
fn take(root: &MountedData) -> Focused {
    let Some(root) = NodeRef::of(root) else {
        return Focused::Unknown;
    };
    match root.read(pressed) {
        None => Focused::Busy,
        Some(None) => Focused::Unknown,
        Some(Some(node)) => match root.write(|doc| {
            doc.set_focus_to(node);
            doc.shell_provider.request_redraw();
        }) {
            Written::Done => Focused::Done,
            Written::Busy => Focused::Busy,
        },
    }
}

/// The element a press at the hover node gives the keyboard to when its click's default was
/// prevented: the nearest focusable one from the pressed element up. `None` when a field has
/// the keyboard, or the press landed in a field or on a disabled control, which keep the focus
/// where Blitz put it.
fn pressed(doc: &BaseDocument) -> Option<NodeId> {
    let focused = doc.get_focussed_node_id().and_then(|id| doc.get_node(id));
    if focused.is_some_and(edits) {
        return None;
    }
    let target = doc.get_hover_node_id()?;
    let chain: Vec<&Node> = std::iter::successors(doc.get_node(target), |node| {
        node.parent.and_then(|id| doc.get_node(id))
    })
    .collect();
    let at = chain.iter().position(|node| node.is_focussable())?;
    let on_the_way = &chain[..=at];
    if on_the_way.iter().any(|node| disabled(node) || edits(node)) {
        return None;
    }
    Some(chain[at].id)
}

/// Whether an element edits text: a Blitz text field, or an editable surface (`role=textbox`,
/// quire's `EditSurface`).
fn edits(node: &Node) -> bool {
    node.element_data().is_some_and(|element| {
        element.text_input_data().is_some()
            || element.attr(LocalName::from("role")) == Some("textbox")
            || element.attr(LocalName::from("contenteditable")).is_some()
    })
}

/// Whether an element is disabled.
fn disabled(node: &Node) -> bool {
    node.element_data()
        .is_some_and(|element| element.attr(LocalName::from("disabled")).is_some())
}

/// What a click at the document's hover node will do to the focus, read through `root`.
fn fallback(root: &MountedData) -> Fallback {
    let Some(root) = NodeRef::of(root) else {
        return Fallback::Renderer;
    };
    let Some(Some((ancestor, candidates))) = root.read(|doc| {
        ancestor_of_click(doc).map(|ancestor| (ancestor, Candidates::from(doc, ancestor)))
    }) else {
        return Fallback::Renderer;
    };
    // Already there: clear it without events, so Blitz's clear has nothing to blur, and give it
    // back after the click.
    if root.read(|doc| doc.get_focussed_node_id() == Some(ancestor)) == Some(true) {
        let _ = root.write(BaseDocument::clear_focus);
    }
    Fallback::Ancestor(Rc::new(MountedData::new(ChainNode {
        doc: root.doc.clone(),
        candidates,
    })))
}

/// Focus the first candidate still in the document if the focus is nowhere: a handler that
/// moved it during the click wins, and one that removed the ancestor sends it further up.
/// Liveness is read now, as the focus is written, not when the click asked.
fn restore(target: &MountedData) -> Focused {
    let Some(target) = Target::of(target) else {
        return Focused::Unknown;
    };
    let picked = target
        .doc()
        .read(|doc| nowhere(doc).then(|| target.pick(doc)).flatten());
    match picked {
        None => Focused::Busy,
        Some(None) => Focused::Unknown,
        Some(Some(node)) => match target.doc().write(|doc| {
            doc.set_focus_to(node);
            doc.shell_provider.request_redraw();
        }) {
            Written::Done => Focused::Done,
            Written::Busy => Focused::Busy,
        },
    }
}

/// Whether the keyboard is nowhere: no focused node, or the root element.
pub(crate) fn nowhere(doc: &BaseDocument) -> bool {
    let root = doc.try_root_element().map(|root| root.id);
    doc.get_focussed_node_id()
        .is_none_or(|focused| Some(focused) == root)
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
