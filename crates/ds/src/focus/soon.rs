//! Moving keyboard focus into a mounted element, the one focus write the design system does.
//!
//! dioxus-native-dom's `set_focus` borrows its document mutably *when it is called*. A task
//! woken in the same turn as a dirty scope is polled inside `render_immediate`, while the
//! mutation writer holds that borrow, so a focus change made from such a task panicked
//! ("RefCell already borrowed", a palette's field focusing on mount while its
//! results re-rendered). Every focus change goes through [`focus_soon`]: the host's
//! [`FocusHost`](crate::host::parts::FocusHost) answers [`Focused::Busy`] instead, and the change is tried again
//! once that render has ended (`ds_style::busy`), then a frame later.

use crate::focus::select::{Landing, Select};
use crate::host::caret::{CaretOwed, InitialCaret};
use crate::host::document::use_document_host;
use crate::host::focused::Focused;
use crate::host::measure::BUSY_ATTEMPTS;
use dioxus::prelude::*;
use ds_style::busy::wait_out_busy;
use std::rc::Rc;

/// How many tries an owed caret waits for the field's first layout: a few frames, not forever.
const LAYOUT_ATTEMPTS: usize = 40;

/// Give `element` the keyboard from a task of the calling scope, a frame later whenever the
/// document is busy. Best-effort: a renderer without focus still shows the element, and the
/// person can click into it. Call it from a handler or a hook, never from inside a task that
/// the renderer may be polling with its document held (it spawns, it does not focus).
///
/// Public so an app focuses its own element (mailo's `.app` shell after a panel closes) with the
/// same wait for a busy document quire's fields use.
pub fn focus_soon(element: Rc<MountedData>) {
    focus_soon_selecting(element, Select::None);
}

/// As [`focus_soon`], then do `select` with the element's text once the caret is in it:
/// [`Select::All`] selects a field's whole value through the host's
/// [`FocusHost::select`](crate::host::parts::FocusHost::select).
pub fn focus_soon_selecting(element: Rc<MountedData>, select: Select) {
    spawn(async move {
        let _ = focus_selecting(&element, select.into()).await;
    });
}

/// As [`focus_soon_selecting`], for a field whose caller listens for focus.
///
/// The host's write (Blitz's `set_focus_to`) dispatched no `focus` event until the fork's
/// 5c526bd2; now it does, and a surface's own `onfocus` hears it. `told` is for the caller that
/// has no `onfocus` of its own to hear it with: it is called once the host's write has moved the
/// focus. It is not called when the write did not land (no host, an element the host does not
/// own, a document that stayed busy).
///
/// Public so an app that draws its own region (a terminal) focuses it, and runs its focus-in,
/// exactly as `EditSurface` does.
pub fn focus_soon_told(element: Rc<MountedData>, select: Select, told: EventHandler<()>) {
    spawn(async move {
        if focus_selecting(&element, select.into()).await == Focused::Done {
            told.call(());
        }
    });
}

/// As [`focus_soon_selecting`], landing the caret at any [`Landing`] (a field's start or end
/// too), for a surface whose own `onfocus` hears the focus event the host's write raises: `told`
/// is no longer called here (calling it too told the caller twice) and stays in the signature
/// until the callers drop it.
///
/// Public so an app's own field surface lands the caret the way `TextField` does.
pub fn focus_landing_told(element: Rc<MountedData>, landing: Landing, told: EventHandler<()>) {
    let _ = told;
    spawn(async move {
        let _ = focus_selecting(&element, landing).await;
    });
}

/// Move the focus to `element` and do `landing` with its text, in one host write: the caret
/// lands with the focus, so nothing typed between the two can be selected away or overwritten.
/// A busy document or a field not laid out yet retries the whole write; the outcome is the
/// focus's.
pub(crate) async fn focus_selecting(element: &MountedData, landing: Landing) -> Focused {
    let host = use_document_host();
    match landing {
        Landing::Place(caret) => {
            let focused = retry_busy(|| host.focus().focus_placing(element, caret)).await;
            land_owed_caret(element, focused, caret).await;
            focused
        }
        Landing::Leave => {
            let focused = focus_element(element).await;
            // Focusing without a caret still leaves a field with text and no editor yet with its
            // caret at the start once the editor is built, so the next key would land before the
            // text: it owes the caret after the text, as a placed `End` does.
            land_owed_caret(element, focused, InitialCaret::End).await;
            focused
        }
    }
}

/// A field with text focused before its first layout has no editor yet, so nothing can have been
/// typed into it: its caret lands as soon as the editor is built, however many frames the layout
/// takes (`LAYOUT_ATTEMPTS`; a select-all that gave up after `BUSY_ATTEMPTS` left the caret at the
/// start). An empty one owes nothing, so a key typed once its editor is built stays where it went.
/// A document too busy to read counts as owing (the placing retries).
async fn land_owed_caret(element: &MountedData, focused: Focused, caret: InitialCaret) {
    let host = use_document_host();
    if focused == Focused::Done && host.caret().caret_owed(element) == CaretOwed::AfterLayout {
        let _ = retry_for(LAYOUT_ATTEMPTS, || host.caret().place_caret(element, caret)).await;
    }
}

/// Move the focus to `element`, waiting out a busy document for up to `BUSY_ATTEMPTS` frames.
pub(crate) async fn focus_element(element: &MountedData) -> Focused {
    let host = use_document_host();
    retry_busy(|| host.focus().focus(element)).await
}

/// Take the keyboard from `element`, waiting out a busy document for up to `BUSY_ATTEMPTS`
/// frames.
pub(crate) async fn blur_element(element: &MountedData) -> Focused {
    let host = use_document_host();
    retry_busy(|| host.focus().blur(element)).await
}

/// Try a host write until the document is free, for up to `BUSY_ATTEMPTS` tries: the first few
/// as soon as the render that holds it ends, so the write lands in the frame it was asked in
/// (`ds_style::busy`), the rest a frame apart.
pub(crate) async fn retry_busy(write: impl FnMut() -> Focused) -> Focused {
    retry_for(BUSY_ATTEMPTS, write).await
}

/// As [`retry_busy`], for up to `attempts` tries.
async fn retry_for(attempts: usize, mut write: impl FnMut() -> Focused) -> Focused {
    for attempt in 0..attempts {
        match write() {
            Focused::Busy => wait_out_busy(attempt).await,
            tried => return tried,
        }
    }
    Focused::Busy
}
