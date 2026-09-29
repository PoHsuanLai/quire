//! Where the keyboard goes when a surface that took it leaves.
//!
//! On Blitz the focus goes nowhere when the focused element is removed, so the host hands it to
//! a focusable ancestor of what was removed. A floating menu's panel has none of its own (it
//! sits in the overlay layer, outside the app's tree), and the element that opened it is the
//! right place for the keyboard, as a web menu button does. A component that takes the keyboard
//! into such a surface tells the host, once the surface is mounted, which element to give it
//! back to: the host then does so when the surface is removed while it still has the keyboard,
//! after every handler and task of that frame has run, so a pick whose handler focused a field
//! is not overridden and nothing races the removal.

use dioxus::prelude::MountedData;
use std::rc::Rc;

/// The host's record of a hand-back: `(surface, opener)`.
pub type Record = dyn Fn(&MountedData, &MountedData);

/// The host's hand-back seam: `(surface, opener)` records that when `surface` is removed while it
/// has the keyboard, `opener` (or its nearest focusable ancestor) should have it next, if `opener`
/// is still in the document. A host with no loop of its own to watch removals holds none, and the
/// renderer's own focus handling stands.
#[derive(Clone, Default)]
pub struct HandBack(Option<Rc<Record>>);

impl HandBack {
    /// A seam that records into `record`.
    pub fn recording(record: Rc<Record>) -> Self {
        HandBack(Some(record))
    }

    /// Record that `surface` hands the keyboard back to `opener` as it goes.
    pub fn record(&self, surface: &MountedData, opener: &MountedData) {
        if let Some(record) = &self.0 {
            record(surface, opener);
        }
    }
}

impl std::fmt::Debug for HandBack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self.0 {
            Some(_) => "HandBack::recording",
            None => "HandBack::none",
        })
    }
}
