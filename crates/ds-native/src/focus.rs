//! The host's focus writes: keyboard focus moved into (`ds::HostFocus`) and out of
//! (`ds::HostBlur`) an element straight in the Blitz document, answering "busy" instead of
//! panicking when the renderer holds the document; its select-all write (`ds::HostSelect`),
//! which selects a field's value once the caret is in it (mailo Phase B, G6); and its selector
//! lookup (`ds::HostFind`, G8), so an app focuses an element it holds no handle for.
//!
//! dioxus-native-dom's own `set_focus` borrows the document mutably when it is called. A task
//! that dioxus polls inside `render_immediate` (it does, when the task woke in the same turn as
//! a dirty scope) runs while the mutation writer holds that borrow, and the call panicked with
//! "RefCell already borrowed" (sill FINDINGS Q43: a palette's field focusing on mount while its
//! results re-rendered). Probing the document first turns that into `Focused::Busy`, and quire
//! tries again a frame later.
//!
//! `launch` and the harness provide them all; a host that is not `ds_native::launch`
//! (shell-host's surfaces, sill's launcher) calls [`provide`] at the top of its root component,
//! beside `ds_native::measure::provide`, and gets every write but the selector lookup, which
//! needs the document itself.

use crate::node_ref::{DocRef, FoundNode, NodeRef, Written, same};
use blitz_dom::Node;
use dioxus::prelude::*;
use ds::{Focused, Found, HostBlur, HostFind, HostFocus, HostSelect};
use std::rc::Rc;

/// The Blitz focus write, as the `ds::HostFocus` a root provides as context. `launch` and the
/// harness provide it already; another host provides it with [`provide`] or
/// `use_context_provider(|| ds_native::focus::FOCUS)`.
pub const FOCUS: HostFocus = HostFocus(focus);

/// The Blitz blur write, as the `ds::HostBlur` a root provides beside [`FOCUS`]: a
/// `FieldHandle::blur` takes the keyboard from its field.
pub const BLUR: HostBlur = HostBlur(blur);

/// The Blitz select-all write, as the `ds::HostSelect` a root provides beside [`FOCUS`]: a
/// field focused with `FocusRequest::with_select_all` has its whole value selected.
pub const SELECT: HostSelect = HostSelect(select_all);

/// Provide [`FOCUS`], [`BLUR`] and [`SELECT`] to the calling component's subtree. Call it at the
/// top of a root that `ds_native::launch` did not start, before any quire field or menu mounts.
pub fn provide() -> HostFocus {
    use_context_provider(|| SELECT);
    use_context_provider(|| BLUR);
    use_context_provider(|| FOCUS)
}

/// The selector lookup over the document `source` reaches, once it has one.
pub(crate) fn finder(source: impl Fn() -> Option<DocRef> + 'static) -> HostFind {
    HostFind {
        find: Rc::new(move |selector| match source() {
            Some(doc) => lookup(doc, selector),
            None => Found::Busy,
        }),
        same,
    }
}

/// The first element in `doc` matching `selector`, as a handle the focus writes accept.
fn lookup(doc: DocRef, selector: &str) -> Found {
    match doc.read(|read| read.query_selector(selector)) {
        None => Found::Busy,
        Some(Err(_)) => Found::BadSelector,
        Some(Ok(None)) => Found::Missing,
        Some(Ok(Some(node))) => {
            Found::Element(Rc::new(MountedData::new(FoundNode(NodeRef { doc, node }))))
        }
    }
}

/// Give `element` the keyboard, if the document is free and the element is a Blitz node.
fn focus(element: &MountedData) -> Focused {
    let Some(node) = NodeRef::of(element) else {
        return Focused::Unknown;
    };
    match node.read(|doc| doc.get_node(node.node).is_some()) {
        None => Focused::Busy,
        Some(false) => Focused::Unknown,
        Some(true) => done(node.write(|doc| {
            doc.set_focus_to(node.node);
        })),
    }
}

/// Take the keyboard from `element` if it has it: Blitz clears the focus, as a click on nothing
/// focusable does, and dispatches no `blur` (the field's handle says so itself).
fn blur(element: &MountedData) -> Focused {
    let Some(node) = NodeRef::of(element) else {
        return Focused::Unknown;
    };
    match node.read(|doc| doc.get_focussed_node_id() == Some(node.node)) {
        None => Focused::Busy,
        Some(false) => Focused::Unknown,
        Some(true) => done(node.write(|doc| {
            doc.clear_focus();
            doc.shell_provider.request_redraw();
        })),
    }
}

/// Select all of `element`'s text, if the document is free and the element is a Blitz text
/// field. Run after [`focus`] has put the caret in it, so the selection is not collapsed by it.
fn select_all(element: &MountedData) -> Focused {
    let Some(node) = NodeRef::of(element) else {
        return Focused::Unknown;
    };
    let field = match node.read(|doc| doc.get_node(node.node).map_or(Field::Absent, field_of)) {
        None => return Focused::Busy,
        Some(field) => field,
    };
    match field {
        Field::Editable => done(node.write(|doc| {
            doc.with_text_input(node.node, |mut driver| driver.select_all());
            doc.shell_provider.request_redraw();
        })),
        // Blitz builds a field's editor with its first layout; a field focused on mount is
        // there before it, so wait a frame as for a busy document.
        Field::NotLaidOut => Focused::Busy,
        Field::Absent => Focused::Unknown,
    }
}

fn done(written: Written) -> Focused {
    match written {
        Written::Done => Focused::Done,
        Written::Busy => Focused::Busy,
    }
}

/// Whether a node is a text field whose text can be selected yet.
enum Field {
    /// A field with its editor.
    Editable,
    /// An `input` or `textarea` not laid out yet: no editor until the first layout.
    NotLaidOut,
    /// Not a text field, or gone.
    Absent,
}

fn field_of(node: &Node) -> Field {
    let Some(element) = node.element_data() else {
        return Field::Absent;
    };
    if element.text_input_data().is_some() {
        return Field::Editable;
    }
    match element.name.local.as_ref() {
        "input" | "textarea" => Field::NotLaidOut,
        _ => Field::Absent,
    }
}
