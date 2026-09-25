//! The host's edit seam (`ds::HostEdit`) on Blitz: hit-testing, caret and selection rects over
//! an edit surface's own inline layout, the window's IME, the clipboard's HTML, and IME routing
//! (FINDINGS "Edit surface"). Every read answers `Probe::Busy` while the renderer holds the
//! document, as `crate::measure` does.
//!
//! `launch` and the harness provide it; another Blitz host (shell-host's surfaces, sill) calls
//! [`provide`] at the top of its root component, beside `ds_native::focus::provide`, and hands
//! its IME events to the surface itself (it has no `launch` window to hear them in).

use crate::edit_geometry::{caret, selection};
use crate::edit_hit::hit;
use crate::edit_ime::EditListeners;
use crate::edit_locate::resolve;
use crate::edit_tree::segments;
use blitz_dom::{BaseDocument, NodeId};
use dioxus::prelude::*;
use dioxus_native_dom::NodeHandle;
use ds::{
    CapturedPointer, HostEdit, ImeEvent, ImeListener, ImeSwitch, Pasted, Point, Probe, Rect,
    TextPosition, TextRange,
};

/// The Blitz edit seam, as the `ds::HostEdit` a root provides as context.
pub const EDIT: HostEdit = HostEdit {
    hit_test,
    caret_rect,
    selection_rects,
    set_ime,
    set_ime_cursor_area,
    read_clipboard_html,
    listen,
    forget,
    capture,
};

/// Provide [`EDIT`] and the IME routing it registers surfaces with to the calling component's
/// subtree. Call it at the top of a root that `ds_native::launch` did not start.
pub fn provide() -> HostEdit {
    use_context_provider(EditListeners::default);
    use_context_provider(|| EDIT)
}

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

fn hit_test(element: &MountedData, at: Point) -> Probe<TextPosition> {
    read(element, |doc, surface| hit(doc, surface, at))
}

fn caret_rect(element: &MountedData, position: &TextPosition) -> Probe<Rect> {
    read(element, |doc, surface| {
        caret(doc, resolve(doc, surface, position)?)
    })
}

fn selection_rects(element: &MountedData, range: &TextRange) -> Probe<Vec<Rect>> {
    read(element, |doc, surface| {
        let from = resolve(doc, surface, &range.anchor)?;
        let to = resolve(doc, surface, &range.focus)?;
        selection(doc, &segments(doc, surface), from, to)
    })
}

fn set_ime(element: &MountedData, switch: ImeSwitch) -> Probe<()> {
    read(element, |doc, _| {
        doc.shell_provider.set_ime_enabled(switch == ImeSwitch::On);
        Some(())
    })
}

fn set_ime_cursor_area(element: &MountedData, area: Rect) -> Probe<()> {
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

fn read_clipboard_html() -> Option<Pasted> {
    crate::clipboard::read_pasted()
}

fn listen(element: &MountedData, sink: EventHandler<ImeEvent>) -> Probe<ImeListener> {
    let Some(handle) = element.downcast::<NodeHandle>() else {
        return Probe::Unknown;
    };
    match try_consume_context::<EditListeners>() {
        Some(listeners) => Probe::Found(listeners.add(handle.node_id(), sink)),
        None => Probe::Unknown,
    }
}

fn forget(listener: ImeListener) {
    if let Some(listeners) = try_consume_context::<EditListeners>() {
        listeners.remove(listener);
    }
}

fn capture(element: &MountedData, sink: EventHandler<CapturedPointer>) -> Probe<()> {
    if element.downcast::<NodeHandle>().is_none() {
        return Probe::Unknown;
    }
    match try_consume_context::<EditListeners>() {
        Some(listeners) => {
            listeners.capture(sink);
            Probe::Found(())
        }
        None => Probe::Unknown,
    }
}
