//! Retrying an edit-host write while the document is busy: the wait `EditSurface` makes for its
//! IME and focus writes, public so a custom surface (a texture-layer pane that places the IME
//! candidate window) makes the same one.

use crate::host::document::DocumentHost;
use crate::host::measure::BUSY_ATTEMPTS;
use crate::host::parts::EditHost;
use crate::host::probe::Probe;
use dioxus::prelude::*;
use ds_core::time::{FRAME_SLACK, clock::sleep};
use std::rc::Rc;

/// Run a host write against `element` from a task of the calling scope, a frame later whenever
/// the document is busy (the same wait `crate::focus::soon::focus_soon` makes), up to
/// `BUSY_ATTEMPTS` tries. `write` is tried again only on [`Probe::Busy`]; a host with no edit part
/// or an element it does not own ends it quietly. Call it from a handler or a hook: it spawns.
pub fn write_soon(
    host: Rc<dyn DocumentHost>,
    element: Rc<MountedData>,
    write: impl Fn(&dyn EditHost, &MountedData) -> Probe<()> + 'static,
) {
    spawn(async move {
        for _ in 0..BUSY_ATTEMPTS {
            let Some(edit) = host.edit() else {
                return;
            };
            match write(edit, &element) {
                Probe::Busy => sleep(FRAME_SLACK).await,
                Probe::Found(()) | Probe::Unknown => return,
            }
        }
    });
}
