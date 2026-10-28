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
use blitz_dom::{BaseDocument, LocalName, Node, NodeId};
use dioxus::prelude::*;
use ds::host::caret::{Caret, CaretOwed, FieldSelection, InitialCaret, caret_at, stem_end};
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

/// Give `element` the keyboard, if the document is free and the element is a Blitz node, a text
/// field with its caret after its text. An element that already has it is left alone:
/// `set_focus_to` on a focused field puts its caret back at the start (a field focused again by
/// selector typed "nvoicei" for "Invoice").
pub(crate) fn focus(element: &MountedData) -> Focused {
    let Some(node) = NodeRef::of(element) else {
        return Focused::Unknown;
    };
    let state = node.read(|doc| match doc.get_node(node.node) {
        None => Held::Gone,
        Some(_) if doc.get_focussed_node_id() == Some(node.node) => Held::Already,
        Some(_) => Held::Not,
    });
    match state {
        None => Focused::Busy,
        Some(Held::Gone) => Focused::Unknown,
        Some(Held::Already) => Focused::Done,
        Some(Held::Not) => done(node.write(|doc| {
            doc.set_focus_to(node.node);
            enable_ime_for_unbuilt_field(doc, node.node);
            caret_after_text(doc, node.node);
        })),
    }
}

/// Give `element` the keyboard and put its caret at `caret` in one write to the document, so the
/// field cannot be typed into between the two (a deferred select-all selected the empty length it
/// had seen and put the caret at 0, and "invoice" became "nvoicei"). A field that has the
/// keyboard already keeps it but takes the caret again (a second request on a focused field selects
/// its text anew; `set_focus_to` is not called, it would put the caret back at the start). A field
/// not laid out yet has no editor to
/// type into or place a caret in, so it takes the keyboard now and its first layout puts the caret
/// at the start; an element that is no text field is only focused.
pub(crate) fn focus_placing(element: &MountedData, caret: InitialCaret) -> Focused {
    let Some(node) = NodeRef::of(element) else {
        return Focused::Unknown;
    };
    let probed = node.read(|doc| {
        doc.get_node(node.node).map(|found| {
            let held = if doc.get_focussed_node_id() == Some(node.node) {
                Held::Already
            } else {
                Held::Not
            };
            (held, field_of(found))
        })
    });
    match probed {
        None => Focused::Busy,
        Some(None | Some((Held::Gone, _))) => Focused::Unknown,
        Some(Some((Held::Already, Field::Editable))) => place_caret(element, caret),
        Some(Some((Held::Already, Field::NotLaidOut | Field::Absent))) => Focused::Done,
        Some(Some((Held::Not, Field::Editable))) => done(node.write(|doc| {
            doc.set_focus_to(node.node);
            put_caret(doc, node.node, caret);
            doc.shell_provider.request_redraw();
        })),
        Some(Some((Held::Not, Field::NotLaidOut | Field::Absent))) => focus(element),
    }
}

/// Put the caret of the text field `node` at `caret`.
fn put_caret(doc: &mut BaseDocument, node: NodeId, caret: InitialCaret) {
    let stem = doc
        .get_node(node)
        .and_then(Node::element_data)
        .and_then(|element| element.text_input_data())
        .map_or(0, |input| stem_end(input.editor.raw_text()));
    doc.with_text_input(node, |mut driver| match caret {
        InitialCaret::SelectAll => driver.select_all(),
        InitialCaret::SelectStem => driver.select_byte_range(0, stem),
        InitialCaret::End => driver.move_to_text_end(),
        InitialCaret::Start => driver.move_to_text_start(),
    });
}

/// Whether a node has the keyboard already.
enum Held {
    /// The node is not in the document.
    Gone,
    /// It has the focus.
    Already,
    /// It is there and does not.
    Not,
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
    let Some(field) = node.read(|doc| doc.get_node(node.node).map_or(Field::Absent, field_of))
    else {
        return Focused::Busy;
    };
    match field {
        Field::Editable => done(node.write(|doc| {
            put_caret(doc, node.node, caret);
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
        Some(caret_at(
            input.editor.raw_text(),
            selection.focus().index(),
            selection.is_collapsed(),
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
                .map_or_else(|| text.chars().count(), |s| s.chars().count())
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

/// Whether `element` is a field with text whose editor is not built yet, so a caret asked for when
/// it was focused is still owed. A document busy rendering is read as owing: the caller then
/// places the caret with a retry, which waits for the editor, rather than leaving a field that is
/// not laid out yet with its caret at the start.
pub(crate) fn caret_owed(element: &MountedData) -> CaretOwed {
    let Some(node) = NodeRef::of(element) else {
        return CaretOwed::No;
    };
    let read = node.read(|doc| {
        let found = doc.get_node(node.node)?;
        let element = found.element_data()?;
        let empty = element
            .attr(LocalName::from("value"))
            .is_none_or(str::is_empty);
        match (field_of(found), empty) {
            (Field::NotLaidOut, false) => Some(CaretOwed::AfterLayout),
            _ => Some(CaretOwed::No),
        }
    });
    match read {
        None => CaretOwed::AfterLayout,
        Some(owed) => owed.unwrap_or(CaretOwed::No),
    }
}

/// Put the caret of `node`, if it is a laid-out text field, after its text. Blitz builds a field's
/// editor with the caret at the start, so a field mounted anew and given the keyboard from code (a
/// remounted field the focus is handed back to) would take the next key before its text: the
/// caret goes after it instead, as on macOS.
pub(crate) fn caret_after_text(doc: &mut BaseDocument, node: NodeId) {
    if doc
        .get_node(node)
        .is_some_and(|found| matches!(field_of(found), Field::Editable))
    {
        doc.with_text_input(node, |mut driver| driver.move_to_text_end());
    }
}

fn done(written: Written) -> Focused {
    match written {
        Written::Done => Focused::Done,
        Written::Busy => Focused::Busy,
    }
}

/// Turn the input method on for a text field that took the keyboard before its first layout.
///
/// Blitz switches the input method on when a node with an editor takes the keyboard
/// (`Node::focus`), and builds the editor with the first layout, so a field focused on mount
/// found none and left it off. On macOS that is not only about composing text: with the input
/// method off winit does not run `interpretKeyEvents`, the standard key bindings (Backspace among
/// them, which Blitz leaves to them on macOS) never arrive, and the field takes typing but
/// cannot delete until the keyboard has moved away and back.
fn enable_ime_for_unbuilt_field(doc: &BaseDocument, node: NodeId) {
    if doc
        .get_node(node)
        .is_some_and(|found| matches!(field_of(found), Field::NotLaidOut))
    {
        doc.shell_provider.set_ime_enabled(true);
    }
}

/// Whether a node is a text field whose text can be selected yet.
enum Field {
    /// A field with its editor.
    Editable,
    /// A text field not laid out yet: no editor until the first layout.
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
        "textarea" => Field::NotLaidOut,
        "input" if takes_text(element.attr(LocalName::from("type"))) => Field::NotLaidOut,
        _ => Field::Absent,
    }
}

/// Whether an `input` of `kind` is one Blitz gives an editor (`blitz-dom`'s layout, the types it
/// builds a `TextInputData` for). A checkbox, a button, a range or a file input never has one, so
/// it is never a field waiting for its editor: turning the input method on for it would leave it
/// on, as Blitz turns it off on blur only for a node with an editor.
fn takes_text(kind: Option<&str>) -> bool {
    matches!(
        kind.map(str::to_ascii_lowercase).as_deref(),
        None | Some("text" | "password" | "email" | "number" | "search" | "tel" | "url")
    )
}

#[cfg(test)]
mod tests {
    use super::takes_text;

    #[test]
    fn only_an_input_blitz_gives_an_editor_waits_for_one() {
        let cases = [
            (None, true),
            (Some("text"), true),
            (Some("PASSWORD"), true),
            (Some("email"), true),
            (Some("number"), true),
            (Some("search"), true),
            (Some("tel"), true),
            (Some("url"), true),
            (Some("checkbox"), false),
            (Some("radio"), false),
            (Some("range"), false),
            (Some("button"), false),
            (Some("submit"), false),
            (Some("file"), false),
            (Some("color"), false),
        ];
        for (kind, text) in cases {
            assert_eq!(takes_text(kind), text, "{kind:?}");
        }
    }
}
