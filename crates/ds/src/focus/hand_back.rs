//! Where the keyboard goes when a surface that took it leaves (mailo gaps 7).
//!
//! On Blitz the focus goes nowhere when the focused element is removed, so ds-native hands it to
//! a focusable ancestor of what was removed. A floating menu's panel has none of its own (it
//! sits in the overlay layer, outside the app's tree), and the element that opened it is the
//! right place for the keyboard, as a web menu button does. A component that takes the keyboard
//! into such a surface tells the host, once the surface is mounted, which element to give it
//! back to: the host then does so when the surface is removed while it still has the keyboard,
//! after every handler and task of that frame has run, so a pick whose handler focused a field
//! is not overridden and nothing races the removal.

use dioxus::prelude::*;
use std::rc::Rc;

/// The host's hand-back seam, provided by ds-native under `FocusFallback::Ancestor` (its
/// default): `(surface, opener)` records that when `surface` is removed while it has the
/// keyboard, `opener` (or its nearest focusable ancestor) should have it next, if `opener` is
/// still in the document. A webview has none: the browser's own focus handling stands.
#[derive(Clone)]
pub struct HostHandBack(pub Rc<Record>);

/// The host's record of a hand-back: `(surface, opener)`.
pub type Record = dyn Fn(&MountedData, &MountedData);

impl std::fmt::Debug for HostHandBack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HostHandBack")
    }
}
