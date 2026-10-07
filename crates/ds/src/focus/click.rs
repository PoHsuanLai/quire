//! Where the keyboard goes after a click on nothing focusable (Native focus, 2026-09-25).
//!
//! A browser moves the focus to the nearest focusable ancestor of what was clicked, so an app
//! whose keys are handled on a `tabindex` shell (mailo's `.app`) keeps hearing them after a
//! click on a row. Blitz instead clears the focus (`events/pointer.rs`, `handle_click`'s
//! "nothing matched"), and every later key goes to the document's root. The host owns the fix:
//! the root's click handler, which runs after every handler inside it, hands the click to
//! the host's [`ClickFocusHost`](crate::host::parts::ClickFocusHost), which a host has unless the app asked
//! for Blitz's own behaviour.
//! A click whose default a component already took (an `EditSurface` focusing itself) is left to
//! that component.
//!
//! **A click a quire control keeps (the rule).** A control that stops a click's propagation (a
//! strip button, a tree row's select button, its trailing slot, any `Propagation::Stop`) keeps
//! it from the root, and one that prevents its default (a tree row's `summary`) makes the root
//! pass it by. Every such control calls [`kept_click`] last in its click handler, so the root's
//! rule still holds: the pressed control, or its nearest focusable ancestor, has the keyboard
//! afterwards, as a pressed button does in a browser. With the default left to run, Blitz clears
//! the focus after the handlers, so the click goes through the host exactly as the root's does.
//! With the default prevented, Blitz leaves the focus where it was, so the host's
//! [`ClickFocusHost::press`](crate::host::parts::ClickFocusHost::press) moves it now, inside the click, and a handler's own later focus (a
//! `focus_soon`) still wins.

use crate::focus::press_focus::{FocusOnPress, on_click};
use crate::focus::soon::retry_busy;
use crate::host::document::{DocumentHost, use_document_host};
use crate::host::fallback::Fallback;
use crate::host::focused::Focused;
use dioxus::prelude::*;
use std::rc::Rc;

/// What a `Ds` root gives the controls inside it for a click they keep: the host, and the root's
/// own element, through which they read the document.
#[derive(Clone)]
pub struct ClickRoot {
    host: Rc<dyn DocumentHost>,
    element: CopyValue<Option<Rc<MountedData>>>,
}

impl std::fmt::Debug for ClickRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClickRoot").finish_non_exhaustive()
    }
}

impl ClickRoot {
    /// The document's host, and the root's element, once mounted.
    pub fn of(element: CopyValue<Option<Rc<MountedData>>>) -> ClickRoot {
        ClickRoot {
            host: use_document_host(),
            element,
        }
    }

    /// The root's own click handler: hand a click nothing inside has taken to the host.
    pub fn clicked(&self, event: &MouseEvent) {
        after_click(&self.host, self.element.peek().clone(), event);
    }
}

/// A click a quire control keeps from the root (it stopped the click's propagation, or
/// prevented its default): the control hands it to the host itself, as the root would have.
/// Call it last in the click handler, after the control's own work, as the root is the last to
/// hear a click. Outside a `Ds` root, or with a host that has no click-focus part
/// (`FocusFallback::BlitzDefault`, a server render, a shell's surface), it does nothing.
pub fn kept_click(event: &MouseEvent) {
    if on_click(event) == FocusOnPress::Refuses {
        return;
    }
    let Some(root) = try_consume_context::<ClickRoot>() else {
        return;
    };
    let element = root.element.peek().clone();
    if event.default_action_enabled() {
        after_click(&root.host, element, event);
    } else {
        press_focus(&root.host, element);
    }
}

/// Give the pressed control the keyboard now, a frame later while the document is busy.
fn press_focus(host: &Rc<dyn DocumentHost>, root: Option<Rc<MountedData>>) {
    let (Some(click), Some(root)) = (host.click_focus(), root) else {
        return;
    };
    if click.press(&root) == Focused::Busy {
        let host = Rc::clone(host);
        spawn(async move {
            if let Some(click) = host.click_focus() {
                let _ = retry_busy(|| click.press(&root)).await;
            }
        });
    }
}

/// The root's click handler: hand a click nothing inside has taken to the host.
fn after_click(host: &Rc<dyn DocumentHost>, root: Option<Rc<MountedData>>, event: &MouseEvent) {
    let (Some(click), Some(root)) = (host.click_focus(), root) else {
        return;
    };
    if !event.default_action_enabled() {
        return;
    }
    if let Fallback::Ancestor(ancestor) = click.fallback(&root) {
        let host = Rc::clone(host);
        spawn(async move {
            if let Some(click) = host.click_focus() {
                let _ = retry_busy(|| click.restore(&ancestor)).await;
            }
        });
    }
}
