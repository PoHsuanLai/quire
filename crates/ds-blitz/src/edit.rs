//! The host's edit seam (`ds::host::parts::EditHost`, `ds::host::parts::ImeHost`) on Blitz: hit-testing, caret and selection
//! rects over an edit surface's own inline layout, the window's IME, and IME routing (FINDINGS
//! "Edit surface"). Every read answers `Probe::Busy` while the renderer holds the document, as
//! `crate::measure` does.
//!
//! `launch` and the harness wire its listeners to their window loop. A shell's surface root has
//! no such loop: its edit surfaces register but are fed no IME events, so its host hands them to
//! the surface itself.

use crate::edit_geometry::{caret, selection};
use crate::edit_hit::hit;
use crate::edit_ime::EditListeners;
use crate::edit_locate::resolve;
use crate::edit_tree::segments;
use blitz_dom::{BaseDocument, NodeId};
use dioxus::prelude::*;
use dioxus_native_dom::NodeHandle;
use ds::host::captured::CapturedPointer;
use ds::host::ime::{ImeEvent, ImeListener, ImeSwitch};
use ds::host::position::{TextPosition, TextRange};
use ds::host::probe::Probe;
use ds::prelude::*;

/// Read the surface's document, if it is free and the element is a Blitz node.
fn read<T>(
    element: &MountedData,
    answer: impl FnOnce(&BaseDocument, NodeId) -> Option<T>,
) -> Probe<T> {
    let Some(handle) = element.downcast::<NodeHandle>() else {
        return Probe::Unknown;
    };
    let Some(doc) = handle.try_doc() else {
        return Probe::Busy;
    };
    if doc.get_node(handle.node_id()).is_none() {
        return Probe::Unknown;
    }
    match answer(&doc, handle.node_id()) {
        Some(found) => Probe::Found(found),
        None => Probe::Unknown,
    }
}

pub(crate) fn hit_test(element: &MountedData, at: Point) -> Probe<TextPosition> {
    read(element, |doc, surface| hit(doc, surface, at))
}

pub(crate) fn caret_rect(element: &MountedData, position: &TextPosition) -> Probe<Rect> {
    read(element, |doc, surface| {
        caret(doc, resolve(doc, surface, position)?)
    })
}

pub(crate) fn selection_rects(element: &MountedData, range: &TextRange) -> Probe<Vec<Rect>> {
    read(element, |doc, surface| {
        let from = resolve(doc, surface, &range.anchor)?;
        let to = resolve(doc, surface, &range.focus)?;
        selection(doc, &segments(doc, surface), from, to)
    })
}

pub(crate) fn set_ime(element: &MountedData, switch: ImeSwitch) -> Probe<()> {
    read(element, |doc, _| {
        doc.shell_provider.set_ime_enabled(switch == ImeSwitch::On);
        Some(())
    })
}

pub(crate) fn set_ime_cursor_area(element: &MountedData, area: Rect) -> Probe<()> {
    read(element, |doc, _| {
        doc.shell_provider.set_ime_cursor_area(
            area.origin.x.0,
            area.origin.y.0,
            area.size.width.0,
            area.size.height.0,
        );
        Some(())
    })
}

pub(crate) fn listen(
    listeners: &EditListeners,
    element: &MountedData,
    sink: EventHandler<ImeEvent>,
) -> Probe<ImeListener> {
    match element.downcast::<NodeHandle>() {
        Some(handle) => Probe::Found(listeners.add(handle.node_id(), sink)),
        None => Probe::Unknown,
    }
}

pub(crate) fn forget(listeners: &EditListeners, listener: ImeListener) {
    listeners.remove(listener);
}

pub(crate) fn capture(
    listeners: &EditListeners,
    element: &MountedData,
    sink: EventHandler<CapturedPointer>,
) -> Probe<()> {
    if element.downcast::<NodeHandle>().is_none() {
        return Probe::Unknown;
    }
    listeners.capture(sink);
    Probe::Found(())
}
