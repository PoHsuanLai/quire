//! A drop target in the app: an element that takes files dragged in from outside the window.

use crate::components::vocab::DropState;
use crate::file_drop::drag::{FileDrag, FileDrop};
use crate::file_drop::host::{DropTarget, HostFileDrop};
use crate::file_drop::track::TargetView;
use dioxus::core::current_scope_id;
use dioxus::prelude::*;

/// A drop target's handle: bind its element with [`FileDropHandle::mounted`], read what the drag
/// looks like from it with [`FileDropHandle::drag`], and write [`FileDropHandle::drop_attr`] as
/// its `data-drop` so CSS can light it.
#[derive(Clone, Copy)]
pub struct FileDropHandle {
    view: Signal<TargetView>,
    owner: ScopeId,
    host: CopyValue<Option<HostFileDrop>>,
    ondrop: Callback<FileDrop>,
}

impl std::fmt::Debug for FileDropHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileDropHandle")
            .field("owner", &self.owner)
            .finish_non_exhaustive()
    }
}

/// Make the calling component's element a drop target for files dragged in from outside the
/// window. `ondrop` hears the paths let go on it (on it or on anything inside it that is not a
/// target of its own).
///
/// ```ignore
/// let drop = ds::use_file_drop(move |files: ds::FileDrop| attach(files.paths));
/// rsx! {
///     div {
///         class: "composer",
///         "data-drop": drop.drop_attr(),
///         onmounted: move |event| drop.mounted(event),
///     }
/// }
/// ```
///
/// Without a host seam (a webview, a server render) the target never lights and never hears a
/// drop.
pub fn use_file_drop(ondrop: impl FnMut(FileDrop) + 'static) -> FileDropHandle {
    let ondrop = use_callback(ondrop);
    let view = use_signal(TargetView::default);
    let owner = use_hook(current_scope_id);
    let host = use_hook(|| CopyValue::new(try_consume_context::<HostFileDrop>()));
    use_drop(move || {
        if let Some(host) = host.peek().as_ref() {
            host.leave(owner);
        }
    });
    FileDropHandle {
        view,
        owner,
        host,
        ondrop,
    }
}

impl FileDropHandle {
    /// Bind the target's element: its `onmounted` handler.
    pub fn mounted(&self, event: MountedEvent) {
        if let Some(host) = self.host.peek().as_ref() {
            host.enter(DropTarget {
                owner: self.owner,
                element: event.data(),
                view: self.view,
                ondrop: self.ondrop,
            });
        }
    }

    /// The drag as this target sees it. Reading it re-renders the caller when it changes.
    pub fn drag(&self) -> FileDrag {
        self.view.read().drag.clone()
    }

    /// Where this target stands in the drag: `Target` with files over it, `Accepts` while files
    /// are dragged elsewhere in the window, `Idle` otherwise.
    pub fn place(&self) -> DropState {
        self.view.read().place
    }

    /// The target's `data-drop`: `target` with files over it, `accepts` while files are dragged
    /// elsewhere in the window, nothing otherwise (the attribute a quire drop place writes).
    pub fn drop_attr(&self) -> Option<&'static str> {
        self.place().drop_attr()
    }
}
