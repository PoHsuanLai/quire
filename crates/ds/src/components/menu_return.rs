//! A floating menu that took the keyboard gives it back to the element that opened it when it
//! closes (mailo gaps 7), as a web menu button does: on Blitz the panel's removal otherwise
//! left the focus nowhere, and an app whose keys are handled on `.app[tabindex]` heard nothing
//! after Escape.
//!
//! The opener is the anchor when it is a mounted element (`Anchor::Mounted`). The panel tells
//! the host as it mounts ([`HostHandBack`]); the host gives the keyboard to the opener (or its
//! nearest focusable ancestor) when the panel is removed while it still has it. A menu anchored
//! at a point or a rect names no element: ds-native then hands the keyboard to the element
//! focused before the menu took it, or to where the pointer pressed to open it.

use crate::components::flow::Flow;
use crate::components::menu_cursor::Cursor;
use crate::focus::HostHandBack;
use crate::geometry::{Anchor, MountedRef};
use dioxus::prelude::*;
use std::rc::Rc;

/// Record, for the host, that `panel` gives the keyboard back to `anchor`'s element: only for a
/// floating menu that takes the keyboard, anchored to an element, under a host that hands back.
pub(crate) fn hand_back(panel: &MountedData, anchor: &Anchor, flow: Flow, active: Cursor) {
    let Anchor::Mounted(MountedRef(opener)) = anchor else {
        return;
    };
    if !active.takes_focus() || flow != Flow::Floating {
        return;
    }
    if let Some(HostHandBack(record)) = try_consume_context::<HostHandBack>() {
        record(panel, Rc::as_ref(opener));
    }
}
