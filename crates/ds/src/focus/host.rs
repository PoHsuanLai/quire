//! Moving keyboard focus into a mounted element, the one focus write the design system does.
//!
//! dioxus-native-dom's `set_focus` borrows its document mutably *when it is called*. A task
//! woken in the same turn as a dirty scope is polled inside `render_immediate`, while the
//! mutation writer holds that borrow, so a focus change made from such a task panicked
//! ("RefCell already borrowed", sill FINDINGS Q43: a palette's field focusing on mount while
//! its results re-rendered). Every focus change goes through [`focus_soon`]: the host's
//! [`HostFocus`] answers [`Focused::Busy`] instead, and with no host the call is guarded
//! (`crate::guarded`) so the collision is `Busy` too; the change is tried again a frame later.

use crate::focus::select::{HostSelect, Select};
use crate::geometry::measure::BUSY_ATTEMPTS;
use crate::guarded::guarded_call;
use crate::time::{FRAME_SLACK, sleep};
use dioxus::prelude::*;
use std::rc::Rc;

/// One attempt at moving the focus through the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Focused {
    /// The element has the focus.
    Done,
    /// The document is busy (rendering); try again next frame.
    Busy,
    /// The host cannot focus this element (not its node, or gone).
    Unknown,
}

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
        let _ = focus_selecting(&element, select).await;
    });
}

/// As [`focus_soon_selecting`], then `told` once a host's write has moved the focus.
///
/// A host's write (`HostFocus`, Blitz's `set_focus_to`) dispatches no `focus` event, so a field
/// whose caller listens for focus would never hear that the seam put the caret in it. Without a
/// host the renderer's own `set_focus` fires the element's real `focus` event, which the field
/// already forwards, so `told` is not called and the caller hears it once.
pub(crate) fn focus_soon_told(element: Rc<MountedData>, select: Select, told: EventHandler<()>) {
    let hosted = try_consume_context::<HostFocus>().is_some();
    spawn(async move {
        if focus_selecting(&element, select).await == Focused::Done && hosted {
            told.call(());
        }
    });
}

/// Move the focus to `element`, then do `select` with its text; the focus's outcome.
pub(crate) async fn focus_selecting(element: &MountedData, select: Select) -> Focused {
    let focused = focus_element(element).await;
    if focused == Focused::Done
        && select == Select::All
        && let Some(HostSelect(select_all)) = try_consume_context::<HostSelect>()
    {
        let _ = retry_busy(|| select_all(element)).await;
    }
    focused
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
            for _ in 0..BUSY_ATTEMPTS {
                match unhosted(element, toward).await {
                    Focused::Busy => sleep(FRAME_SLACK).await,
                    tried => return tried,
                }
            }
            Focused::Busy
        }
    }
}

/// Try a host write until the document is free, for up to `BUSY_ATTEMPTS` frames.
pub(crate) async fn retry_busy(mut write: impl FnMut() -> Focused) -> Focused {
    for _ in 0..BUSY_ATTEMPTS {
        match write() {
            Focused::Busy => sleep(FRAME_SLACK).await,
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
