//! The host's focus writes: keyboard focus moved into (`ds::host::parts::FocusHost::focus`) and out of
//! (`blur`) an element straight in the Blitz document, answering "busy" instead of panicking when
//! the renderer holds the document; its select-all write, which selects a field's value once the
//! caret is in it; the caret reads and writes (`ds::host::parts::CaretHost`); and its selector lookup, so an
//! app focuses an element it holds no handle for.
//!
//! dioxus-native-dom's own `set_focus` borrows the document mutably when it is called. A task
//! that dioxus polls inside `render_immediate` (it does, when the task woke in the same turn as
//! a dirty scope) runs while the mutation writer holds that borrow, and the call panicked with
//! "RefCell already borrowed" (a palette's field focusing on mount while its
//! results re-rendered). Probing the document first turns that into `Focused::Busy`, and quire
//! tries again a frame later.

use crate::blitz_host::FindDocument;
use crate::node_ref::{DocRef, FoundNode, NodeRef, Written};
use blitz_dom::Node;
use dioxus::prelude::*;
use ds::host::caret::{Caret, Collapsed, FieldSelection, InitialCaret, caret_at};
use ds::host::found::Found;
use ds::prelude::*;
use std::rc::Rc;

/// The selector lookup over the document `source` reaches, once it has one.
pub fn finder(source: impl Fn() -> Option<DocRef> + 'static) -> FindDocument {
    Rc::new(move |selector| match source() {
        Some(doc) => lookup(doc, selector),
        None => Found::Busy,
    })
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
pub(crate) fn focus(element: &MountedData) -> Focused {
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
pub(crate) fn blur(element: &MountedData) -> Focused {
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
pub(crate) fn select_all(element: &MountedData) -> Focused {
    place_caret(element, InitialCaret::SelectAll)
}

/// Put the caret of `element`'s text field at `caret`, if the document is free and the element is
/// a Blitz text field. Run after [`focus`] has put the caret in it.
pub(crate) fn place_caret(element: &MountedData, caret: InitialCaret) -> Focused {
    let Some(node) = NodeRef::of(element) else {
        return Focused::Unknown;
    };
    let field = match node.read(|doc| doc.get_node(node.node).map_or(Field::Absent, field_of)) {
        None => return Focused::Busy,
        Some(field) => field,
    };
    match field {
        Field::Editable => done(node.write(|doc| {
            doc.with_text_input(node.node, |mut driver| match caret {
                InitialCaret::SelectAll => driver.select_all(),
                InitialCaret::End => driver.move_to_text_end(),
                InitialCaret::Start => driver.move_to_text_start(),
            });
            doc.shell_provider.request_redraw();
        })),
        // Blitz builds a field's editor with its first layout; a field focused on mount is
        // there before it, so wait a frame as for a busy document.
        Field::NotLaidOut => Focused::Busy,
        Field::Absent => Focused::Unknown,
    }
}

/// Where the caret is in `element`'s text field: read from the field's editor, which a key
/// handler may do (Blitz holds no borrow of the document while a handler runs). A node that is
/// not a laid-out field, or a document busy rendering, reads `Unknown`.
pub(crate) fn caret(element: &MountedData) -> Caret {
    let Some(node) = NodeRef::of(element) else {
        return Caret::Unknown;
    };
    node.read(|doc| {
        let input = doc.get_node(node.node)?.element_data()?.text_input_data()?;
        let selection = input.editor.raw_selection();
        let collapsed = if selection.is_collapsed() {
            Collapsed::Yes
        } else {
            Collapsed::No
        };
        Some(caret_at(
            input.editor.raw_text(),
            selection.focus().index(),
            collapsed,
        ))
    })
    .flatten()
    .unwrap_or(Caret::Unknown)
}

/// Whether `element`'s field has the keyboard and where its selection is, in characters of its
/// text. A node that is not a laid-out field, or a document busy rendering, reads `Unknown`.
pub(crate) fn selection(element: &MountedData) -> FieldSelection {
    let Some(node) = NodeRef::of(element) else {
        return FieldSelection::Unknown;
    };
    node.read(|doc| {
        let field = doc.get_node(node.node)?;
        if !field.is_focussed() {
            return Some(FieldSelection::Unfocused);
        }
        let editor = &field.element_data()?.text_input_data()?.editor;
        let text = editor.raw_text();
        let chars = |byte: usize| {
            text.get(..byte)
                .map_or(text.chars().count(), |s| s.chars().count())
        };
        let selection = editor.raw_selection();
        Some(FieldSelection::Focused {
            anchor: chars(selection.anchor().index()),
            focus: chars(selection.focus().index()),
        })
    })
    .flatten()
    .unwrap_or(FieldSelection::Unknown)
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
