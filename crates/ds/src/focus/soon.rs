//! Moving keyboard focus into a mounted element, the one focus write the design system does.
//!
//! dioxus-native-dom's `set_focus` borrows its document mutably *when it is called*. A task
//! woken in the same turn as a dirty scope is polled inside `render_immediate`, while the
//! mutation writer holds that borrow, so a focus change made from such a task panicked
//! ("RefCell already borrowed", a palette's field focusing on mount while its
//! results re-rendered). Every focus change goes through [`focus_soon`]: the host's
//! [`FocusHost`](crate::FocusHost) answers [`Focused::Busy`] instead, and the change is tried again
//! once that render has ended (`crate::core::busy`), then a frame later.

use crate::core::busy::wait_out_busy;
use crate::focus::select::{Landing, Select};
use crate::host::caret::InitialCaret;
use crate::host::document::use_document_host;
use crate::host::focused::Focused;
use crate::host::measure::BUSY_ATTEMPTS;
use dioxus::prelude::*;
use std::rc::Rc;

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
/// [`FocusHost::select`](crate::FocusHost::select).
pub fn focus_soon_selecting(element: Rc<MountedData>, select: Select) {
    spawn(async move {
        let _ = focus_selecting(&element, select.into()).await;
    });
}

/// As [`focus_soon_selecting`], then `told` once the host's write has moved the focus.
///
/// The host's write (Blitz's `set_focus_to`) dispatches no `focus` event, so a field whose caller
/// listens for focus would never hear that the seam put the caret in it.
pub(crate) fn focus_soon_told(element: Rc<MountedData>, landing: Landing, told: EventHandler<()>) {
    spawn(async move {
        if focus_selecting(&element, landing).await == Focused::Done {
            told.call(());
        }
    });
}

/// Move the focus to `element`, then do `landing` with its text; the focus's outcome.
pub(crate) async fn focus_selecting(element: &MountedData, landing: Landing) -> Focused {
    let focused = focus_element(element).await;
    if focused == Focused::Done
        && let Landing::Place(caret) = landing
    {
        let _ = place_caret(element, caret).await;
    }
    focused
}

/// Put the caret of the focused field `element` at `caret`, through the host: a whole-value
/// selection through [`FocusHost::select`](crate::FocusHost::select), an end through
/// [`CaretHost::place_caret`](crate::CaretHost::place_caret).
async fn place_caret(element: &MountedData, caret: InitialCaret) -> Focused {
    let host = use_document_host();
    match caret {
        InitialCaret::SelectAll => retry_busy(|| host.focus().select(element)).await,
        InitialCaret::End | InitialCaret::Start => {
            retry_busy(|| host.caret().place_caret(element, caret)).await
        }
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
/// (`crate::core::busy`), the rest a frame apart.
pub(crate) async fn retry_busy(mut write: impl FnMut() -> Focused) -> Focused {
    for attempt in 0..BUSY_ATTEMPTS {
        match write() {
            Focused::Busy => wait_out_busy(attempt).await,
            tried => return tried,
        }
    }
    Focused::Busy
}
