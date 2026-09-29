//! Moving keyboard focus into a mounted element, the one focus write the design system does.
//!
//! dioxus-native-dom's `set_focus` borrows its document mutably *when it is called*. A task
//! woken in the same turn as a dirty scope is polled inside `render_immediate`, while the
//! mutation writer holds that borrow, so a focus change made from such a task panicked
//! ("RefCell already borrowed", a palette's field focusing on mount while
//! its results re-rendered). Every focus change goes through [`focus_soon`]: the host's
//! [`HostFocus`] answers [`Focused::Busy`] instead, and with no host the call is guarded
//! (`crate::core::guarded`) so the collision is `Busy` too; the change is tried again once that
//! render
//! has ended (`crate::core::busy`), then a frame later.

use crate::core::busy::wait_out_busy;
use crate::core::guarded::guarded_call;
use crate::focus::caret::HostPlaceCaret;
use crate::focus::select::{HostSelect, Landing, Select};
use crate::host::caret::InitialCaret;
use crate::host::focused::Focused;
use crate::host::measure::BUSY_ATTEMPTS;
use dioxus::prelude::*;
use std::rc::Rc;

/// The host's own focus write, provided as root context by `ds-native` (`ds_native::launch`,
/// its harness, and `ds_native::focus::provide` for any other Blitz host); without one, focus
/// goes through `MountedData::set_focus`, guarded so a renderer that holds its document answers
/// [`Focused::Busy`] instead of panicking.
#[derive(Debug, Clone, Copy)]
pub struct HostFocus(pub fn(&MountedData) -> Focused);

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
/// [`Select::All`] selects a field's whole value through the host's [`HostSelect`].
pub fn focus_soon_selecting(element: Rc<MountedData>, select: Select) {
    spawn(async move {
        let _ = focus_selecting(&element, select.into()).await;
    });
}

/// As [`focus_soon_selecting`], then `told` once a host's write has moved the focus.
///
/// A host's write (`HostFocus`, Blitz's `set_focus_to`) dispatches no `focus` event, so a field
/// whose caller listens for focus would never hear that the seam put the caret in it. Without a
/// host the renderer's own `set_focus` fires the element's real `focus` event, which the field
/// already forwards, so `told` is not called and the caller hears it once.
pub(crate) fn focus_soon_told(element: Rc<MountedData>, landing: Landing, told: EventHandler<()>) {
    let hosted = try_consume_context::<HostFocus>().is_some();
    spawn(async move {
        if focus_selecting(&element, landing).await == Focused::Done && hosted {
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
/// selection through its [`HostSelect`], an end through its [`HostPlaceCaret`]. Without the
/// host's write the caret stays where the renderer put it ([`Focused::Unknown`]).
async fn place_caret(element: &MountedData, caret: InitialCaret) -> Focused {
    match caret {
        InitialCaret::SelectAll => match try_consume_context::<HostSelect>() {
            Some(HostSelect(select_all)) => retry_busy(|| select_all(element)).await,
            None => Focused::Unknown,
        },
        InitialCaret::End | InitialCaret::Start => match try_consume_context::<HostPlaceCaret>() {
            Some(HostPlaceCaret(place)) => retry_busy(|| place(element, caret)).await,
            None => Focused::Unknown,
        },
    }
}

/// Move the focus to `element`, waiting out a busy document for up to `BUSY_ATTEMPTS` frames.
pub(crate) async fn focus_element(element: &MountedData) -> Focused {
    let host = try_consume_context::<HostFocus>().map(|HostFocus(focus)| focus);
    write(element, host, Toward::In).await
}

/// The host's write that takes the keyboard from an element, provided beside [`HostFocus`] by
/// `ds-native`: [`Focused::Done`] when the element had the focus and no longer has it,
/// [`Focused::Unknown`] when it did not have it. Without one, the blur goes through
/// `MountedData::set_focus(false)`, guarded as [`focus_soon`] guards a focus.
#[derive(Debug, Clone, Copy)]
pub struct HostBlur(pub fn(&MountedData) -> Focused);

/// Take the keyboard from `element`, waiting out a busy document for up to `BUSY_ATTEMPTS`
/// frames.
pub(crate) async fn blur_element(element: &MountedData) -> Focused {
    let host = try_consume_context::<HostBlur>().map(|HostBlur(blur)| blur);
    write(element, host, Toward::Out).await
}

/// Which way a focus write moves the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Toward {
    /// Into the element.
    In,
    /// Out of it.
    Out,
}

/// A focus write through `host`, else through the renderer (guarded), retried while busy.
async fn write(
    element: &MountedData,
    host: Option<fn(&MountedData) -> Focused>,
    toward: Toward,
) -> Focused {
    match host {
        Some(host) => retry_busy(|| host(element)).await,
        None => {
            for attempt in 0..BUSY_ATTEMPTS {
                match unhosted(element, toward).await {
                    Focused::Busy => wait_out_busy(attempt).await,
                    tried => return tried,
                }
            }
            Focused::Busy
        }
    }
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

/// A focus change with no host seam: the call and its future guarded, a panic read as busy.
async fn unhosted(element: &MountedData, toward: Toward) -> Focused {
    match guarded_call(|| element.set_focus(toward == Toward::In)).await {
        Some(Ok(())) => Focused::Done,
        Some(Err(_)) => Focused::Unknown,
        None => Focused::Busy,
    }
}
