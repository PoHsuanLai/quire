//! Where the keyboard goes after a click on nothing focusable (Native focus, 2026-09-25).
//!
//! A browser moves the focus to the nearest focusable ancestor of what was clicked, so an app
//! whose keys are handled on a `tabindex` shell (mailo's `.app`) keeps hearing them after a
//! click on a row. Blitz instead clears the focus (`events/pointer.rs`, `handle_click`'s
//! "nothing matched"), and every later key goes to the document's root. The host owns the fix:
//! the root's click handler, which runs after every handler inside it, hands the click to
//! [`HostClickFocus`], which ds-native provides unless the app asked for Blitz's own behaviour.
//! A click whose default a component already took (an `EditSurface` focusing itself) is left to
//! that component.
//!
//! **A click a quire control keeps (the rule).** A control that stops a click's propagation (a
//! strip button, a tree row's select button, its trailing slot, any `Propagation::Stop`) keeps
//! it from the root, and one that prevents its default (a tree row's `summary`) makes the root
//! pass it by. Every such control calls [`kept_click`] last in its click handler, so the root's
//! rule still holds: the pressed control, or its nearest focusable ancestor, has the keyboard
//! afterwards, as a pressed button does in a browser. With the default left to run, Blitz clears
//! the focus after the handlers, so the click goes through [`HostClickFocus`] exactly as the
//! root's does. With the default prevented, Blitz leaves the focus where it was, so the host's
//! [`HostPressFocus`] moves it now, inside the click, and a handler's own later focus (a
//! `focus_soon`) still wins.

use crate::focus::host::{Focused, retry_busy};
use dioxus::prelude::*;
use std::rc::Rc;

/// What the host found for a click.
#[derive(Clone)]
pub enum Fallback {
    /// The renderer's default stands: it focuses what was clicked, or it keeps the focus, or
    /// there is no focusable ancestor to move it to.
    Renderer,
    /// The renderer is about to clear the focus; this ancestor should have it once the click is
    /// done.
    Ancestor(Rc<MountedData>),
}

impl std::fmt::Debug for Fallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Fallback::Renderer => "Renderer",
            Fallback::Ancestor(_) => "Ancestor",
        })
    }
}

/// The host's click-focus seam, provided as root context by ds-native under
/// `FocusFallback::Ancestor` (its default): `fallback` reads the click from the document through
/// any element of it (the root's), and `restore` gives the ancestor the keyboard once the click's
/// default has run, only if the click left the focus nowhere (a handler that moved it wins).
#[derive(Debug, Clone, Copy)]
pub struct HostClickFocus {
    /// Called from the root's click handler, before the renderer's default action.
    pub fallback: fn(&MountedData) -> Fallback,
    /// Called a frame later with the ancestor `fallback` found. It checks that the ancestor is
    /// still in the document when it focuses, not when the click asked: a click whose handler
    /// removed it (a "Show images" button that goes once pressed) sends the keyboard to the
    /// next focusable ancestor instead (mailo gaps 7).
    pub restore: fn(&MountedData) -> Focused,
}

/// The host's focus write for a press whose click a quire control kept with its default
/// prevented, provided as root context by ds-native beside [`HostClickFocus`] (under
/// `FocusFallback::Ancestor`). It reads the press from the document through any element of it
/// (the root's) and gives the keyboard to the nearest focusable element from the pressed one up,
/// now. It leaves the focus alone when a text field or editable surface has it (the click
/// cannot send that field a `blur`), and when the press landed in a field or on a disabled
/// control.
#[derive(Debug, Clone, Copy)]
pub struct HostPressFocus(pub fn(&MountedData) -> Focused);

/// What a `Ds` root gives the controls inside it for a click they keep: the host's seams, if the
/// host provided them, and the root's own element, through which they read the document.
#[derive(Clone, Copy)]
pub(crate) struct ClickRoot {
    host: Option<HostClickFocus>,
    press: Option<HostPressFocus>,
    element: CopyValue<Option<Rc<MountedData>>>,
}

impl ClickRoot {
    /// The root's seams, read from the host's context, and its element, once mounted.
    pub(crate) fn of(element: CopyValue<Option<Rc<MountedData>>>) -> ClickRoot {
        ClickRoot {
            host: try_consume_context::<HostClickFocus>(),
            press: try_consume_context::<HostPressFocus>(),
            element,
        }
    }

    /// The root's own click handler: hand a click nothing inside has taken to the host.
    pub(crate) fn clicked(&self, event: &MouseEvent) {
        after_click(self.host, self.element.peek().clone(), event);
    }
}

/// A click a quire control keeps from the root (it stopped the click's propagation, or
/// prevented its default): the control hands it to the host itself, as the root would have.
/// Call it last in the click handler, after the control's own work, as the root is the last to
/// hear a click. Outside a `Ds` root, or without the host's seams (`FocusFallback::BlitzDefault`,
/// a server render, the web), it does nothing.
pub(crate) fn kept_click(event: &MouseEvent) {
    let Some(root) = try_consume_context::<ClickRoot>() else {
        return;
    };
    let element = root.element.peek().clone();
    if event.default_action_enabled() {
        after_click(root.host, element, event);
    } else {
        press_focus(root.press, element);
    }
}

/// Give the pressed control the keyboard now, a frame later while the document is busy.
fn press_focus(press: Option<HostPressFocus>, root: Option<Rc<MountedData>>) {
    let (Some(HostPressFocus(take)), Some(root)) = (press, root) else {
        return;
    };
    if take(&root) == Focused::Busy {
        spawn(async move {
            let _ = retry_busy(|| take(&root)).await;
        });
    }
}

/// The root's click handler: hand a click nothing inside has taken to the host.
fn after_click(host: Option<HostClickFocus>, root: Option<Rc<MountedData>>, event: &MouseEvent) {
    let (Some(host), Some(root)) = (host, root) else {
        return;
    };
    if !event.default_action_enabled() {
        return;
    }
    if let Fallback::Ancestor(ancestor) = (host.fallback)(&root) {
        spawn(async move {
            let _ = retry_busy(|| (host.restore)(&ancestor)).await;
        });
    }
}
