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

/// The root's click handler: hand a click nothing inside has taken to the host.
pub(crate) fn after_click(
    host: Option<HostClickFocus>,
    root: Option<Rc<MountedData>>,
    event: &MouseEvent,
) {
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
