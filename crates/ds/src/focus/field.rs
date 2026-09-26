//! A caller's handle on one `TextInput` (mailo gaps, G8): give it the keyboard (selecting its
//! text), take the keyboard from it, or read its mounted element, from any handler. The
//! `TextInput`'s own `Focus::Controlled` asks for the focus by re-rendering; a handle acts at
//! once, and hands out the element, which a request never did.

use crate::focus::host::{blur_element, focus_selecting};
use crate::focus::select::Select;
use crate::focus::targets::FocusTarget;
use crate::focus::{Focused, HostFocus};
use dioxus::core::Runtime;
use dioxus::prelude::*;
use std::rc::Rc;

/// A handle on one field: pass it as `TextInput { handle: Some(handle) }`, which fills it as the
/// field mounts. Before that (or after the field unmounts) every call does nothing.
///
/// A focus or blur through a host (Blitz) dispatches no event, so the handle calls the field's
/// `onfocus` or `onblur` itself once the write lands; without a host the renderer's own event
/// fires and the handle stays quiet, so the caller hears each once.
#[derive(Clone, Copy)]
pub struct FieldHandle {
    field: Signal<Option<FocusTarget>>,
    /// The scope that made the handle: the focus task runs there, so a handler of a component
    /// that is closing (a menu handing the keyboard back) still lands it.
    owner: ScopeId,
}

/// The same handle is the same field.
impl PartialEq for FieldHandle {
    fn eq(&self, other: &Self) -> bool {
        self.field == other.field
    }
}

impl std::fmt::Debug for FieldHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldHandle")
            .field("owner", &self.owner)
            .finish_non_exhaustive()
    }
}

/// A field handle owned by the calling component.
pub fn use_field_handle() -> FieldHandle {
    FieldHandle {
        field: use_signal(|| None),
        owner: use_hook(dioxus::core::current_scope_id),
    }
}

impl FieldHandle {
    /// The field has mounted (or re-mounted): the field sets it.
    pub(crate) fn fill(&self, target: FocusTarget) {
        let mut field = self.field;
        let _ = field.try_write().map(|mut slot| *slot = Some(target));
    }

    /// The field's element, once mounted: for `ds::focus_soon`, a measurement or a scroll.
    /// Read in render, it re-renders the reader as the field mounts.
    pub fn element(&self) -> Option<Rc<MountedData>> {
        self.field
            .try_read()
            .ok()
            .and_then(|slot| slot.as_ref().map(|target| Rc::clone(&target.element)))
    }

    /// Give the field the keyboard, a frame later if the document is busy, then do `select`
    /// with its text; the field's `onfocus` hears it once.
    pub fn focus(&self, select: Select) {
        let Some(target) = self.target() else {
            return;
        };
        let hosted = self.hosted();
        self.run(async move {
            if focus_selecting(&target.element, select.into()).await == Focused::Done && hosted {
                target.told.focus.call(());
            }
        });
    }

    /// Take the keyboard from the field, if it has it; its `onblur` (and the commit it makes on
    /// blur) hears it once.
    pub fn blur(&self) {
        let Some(target) = self.target() else {
            return;
        };
        let hosted = self.hosted();
        self.run(async move {
            if blur_element(&target.element).await == Focused::Done && hosted {
                target.told.blur.call(());
            }
        });
    }

    fn target(&self) -> Option<FocusTarget> {
        self.field.try_peek().ok().and_then(|slot| slot.clone())
    }

    /// Whether a host does the writes, so no renderer event will tell the field.
    fn hosted(&self) -> bool {
        Runtime::try_current().is_some_and(|runtime| {
            runtime.in_scope(self.owner, || try_consume_context::<HostFocus>().is_some())
        })
    }

    /// Spawn `work` in the handle's own scope.
    fn run(&self, work: impl Future<Output = ()> + 'static) {
        if let Some(runtime) = Runtime::try_current() {
            runtime.in_scope(self.owner, || {
                spawn(work);
            });
        }
    }
}
