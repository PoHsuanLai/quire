//! The host's focus write (`ds::HostFocus`): keyboard focus moved straight in the Blitz
//! document, answering "busy" instead of panicking when the renderer holds the document; and
//! its select-all write (`ds::HostSelect`), which selects a field's value once the caret is in
//! it (mailo Phase B, G6).
//!
//! dioxus-native-dom's own `set_focus` borrows the document mutably when it is called. A task
//! that dioxus polls inside `render_immediate` (it does, when the task woke in the same turn as
//! a dirty scope) runs while the mutation writer holds that borrow, and the call panicked with
//! "RefCell already borrowed" (sill FINDINGS Q43: a palette's field focusing on mount while its
//! results re-rendered). Probing with `NodeHandle::try_doc` first turns that into
//! `Focused::Busy`, and quire tries again a frame later.
//!
//! `launch` and the harness provide it; a host that is not `ds_native::launch` (shell-host's
//! surfaces, sill's launcher) calls [`provide`] at the top of its root component, beside
//! `ds_native::measure::provide`.

use blitz_dom::Node;
use dioxus::prelude::*;
use dioxus_native_dom::NodeHandle;
use ds::{Focused, HostFocus, HostSelect};
use std::panic::{AssertUnwindSafe, catch_unwind};

/// The Blitz focus write, as the `ds::HostFocus` a root provides as context. `launch` and the
/// harness provide it already; another host provides it with [`provide`] or
/// `use_context_provider(|| ds_native::focus::FOCUS)`.
pub const FOCUS: HostFocus = HostFocus(focus);

/// The Blitz select-all write, as the `ds::HostSelect` a root provides beside [`FOCUS`]: a
/// field focused with `FocusRequest::with_select_all` has its whole value selected.
pub const SELECT: HostSelect = HostSelect(select_all);

/// Provide [`FOCUS`] and [`SELECT`] to the calling component's subtree. Call it at the top of a
/// root that `ds_native::launch` did not start, before any quire field or menu mounts.
pub fn provide() -> HostFocus {
    use_context_provider(|| SELECT);
    use_context_provider(|| FOCUS)
}

/// Give `element` the keyboard, if the document is free and the element is a Blitz node.
fn focus(element: &MountedData) -> Focused {
    let Some(handle) = element.downcast::<NodeHandle>() else {
        return Focused::Unknown;
    };
    match handle.try_doc() {
        None => return Focused::Busy,
        Some(doc) if doc.get_node(handle.node_id()).is_none() => return Focused::Unknown,
        Some(_) => {}
    }
    // Free a moment ago, on this thread: only a shared borrow held further up the stack could
    // refuse the write now, and that is busy too.
    match catch_unwind(AssertUnwindSafe(|| {
        handle.doc_mut().set_focus_to(handle.node_id());
    })) {
        Ok(()) => Focused::Done,
        Err(_) => Focused::Busy,
    }
}

/// Select all of `element`'s text, if the document is free and the element is a Blitz text
/// field. Run after [`focus`] has put the caret in it, so the selection is not collapsed by it.
fn select_all(element: &MountedData) -> Focused {
    let Some(handle) = element.downcast::<NodeHandle>() else {
        return Focused::Unknown;
    };
    let field = match handle.try_doc() {
        None => return Focused::Busy,
        Some(doc) => doc
            .get_node(handle.node_id())
            .map_or(Field::Absent, field_of),
    };
    match field {
        Field::Editable => {}
        // Blitz builds a field's editor with its first layout; a field focused on mount is
        // there before it, so wait a frame as for a busy document.
        Field::NotLaidOut => return Focused::Busy,
        Field::Absent => return Focused::Unknown,
    }
    match catch_unwind(AssertUnwindSafe(|| {
        let mut doc = handle.doc_mut();
        doc.with_text_input(handle.node_id(), |mut driver| driver.select_all());
        doc.shell_provider.request_redraw();
    })) {
        Ok(()) => Focused::Done,
        Err(_) => Focused::Busy,
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
