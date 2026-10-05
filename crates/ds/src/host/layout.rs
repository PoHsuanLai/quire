//! The rect of a mounted element, kept in a signal by the host's frame phase: no read waits for a
//! frame and none retries, because the host publishes the rect after each layout that changed it
//! (`ds::host::phase`). A host that runs no phase answers `Unsupported`, and the rect is read
//! through the host's `measure` instead (see `ds::host::measure`).

use crate::host::document::use_document_host;
use crate::host::measure::{MountedRef, read_after_layout};
use crate::host::phase::{Observe, Observed, Watch};
use crate::host::resized::WindowResized;
use dioxus::prelude::*;
use ds_core::geometry::units::Rect;
use ds_core::time::{FRAME_SLACK, clock::sleep};
use ds_style::task::try_set;

/// How a hook's rect gets its values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Driven {
    /// Published by the host's frame phase.
    ByPhase,
    /// Read through `measure`, after a frame, at mount and at each window resize.
    ByReads,
}

/// `node`'s border box in logical pixels, in the window's coordinates: `None` until the node is
/// mounted and the host has laid it out, then the rect after each layout that changed it. `node`
/// is the typed handle an element's `onmounted` gives (`MountedRef(event.data())`).
pub fn use_layout(node: ReadSignal<Option<MountedRef>>) -> ReadSignal<Option<Rect>> {
    let rect = use_signal(|| None::<Rect>);
    use_layout_into(node, rect);
    rect.into()
}

/// [`use_layout`] writing into a signal the caller owns, which it may also set itself.
pub(crate) fn use_layout_into(node: ReadSignal<Option<MountedRef>>, rect: Signal<Option<Rect>>) {
    let host = use_document_host();
    let mut watch = use_signal(|| None::<Watch>);
    let mut driven = use_signal(|| Driven::ByPhase);
    use_effect(move || {
        let Some(mounted) = node() else {
            watch.set(None);
            return;
        };
        match host.geometry().observe(&mounted.0, Observe::Rect(rect)) {
            Observed::Watching(found) => {
                driven.set(Driven::ByPhase);
                watch.set(Some(found));
            }
            Observed::Unsupported => {
                watch.set(None);
                driven.set(Driven::ByReads);
            }
        }
    });
    let resized = try_consume_context::<WindowResized>();
    use_effect(move || {
        let changes = resized.map_or(0, |resized| resized.count());
        let (Some(mounted), Driven::ByReads) = (node(), driven()) else {
            return;
        };
        // The first read is at mount; a resize is read again after the window's layout follows.
        spawn(async move {
            if changes > 0 {
                sleep(FRAME_SLACK).await;
            }
            if let Some(read) = read_after_layout(&mounted).await {
                let _ = try_set(rect, Some(read));
            }
        });
    });
}
