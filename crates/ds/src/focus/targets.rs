//! The fields a focus from outside can reach, and what each one says when it gets or loses the
//! caret that way.
//!
//! A host's focus write dispatches no `focus` or `blur` event (Blitz's `set_focus_to`), so a field
//! focused by a [`FieldHandle`](crate::FieldHandle) or by
//! [`focus_by_selector`](crate::focus_by_selector) would never hear it. Each mounted
//! `TextField` enters itself here with its own handlers; a focus that lands on it through a host
//! calls them, the same told-path its `Focus::OnMount` takes.

use crate::host::found::SameNode;
use crate::host::parts::GeometryHost;
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// What a field says when a host write moves the caret in or out of it: its `onfocus`, and its
/// `onblur` (with the commit it makes on blur).
#[derive(Clone, Copy)]
pub(crate) struct Told {
    pub(crate) focus: EventHandler<()>,
    pub(crate) blur: EventHandler<()>,
}

/// One mounted field.
#[derive(Clone)]
pub(crate) struct FocusTarget {
    /// The field's scope: the entry's key, and where its focus tasks run.
    pub(crate) owner: ScopeId,
    pub(crate) element: Rc<MountedData>,
    pub(crate) told: Told,
}

/// Every mounted field, kept as a root context the first field provides.
#[derive(Clone, Default)]
pub(crate) struct FocusTargets(Rc<RefCell<Vec<FocusTarget>>>);

impl FocusTargets {
    /// The document's list, provided at the root by the first caller.
    pub(crate) fn root() -> Self {
        try_consume_context::<FocusTargets>()
            .unwrap_or_else(|| dioxus::core::provide_root_context(FocusTargets::default()))
    }

    /// Enter `target`, replacing any earlier entry of its field.
    pub(crate) fn enter(&self, target: FocusTarget) {
        let mut all = self.0.borrow_mut();
        all.retain(|held| held.owner != target.owner);
        all.push(target);
    }

    /// The field in `owner` has unmounted.
    pub(crate) fn leave(&self, owner: ScopeId) {
        self.0.borrow_mut().retain(|held| held.owner != owner);
    }

    /// What the field at `element` says, if `element` is one: the host's node identity decides,
    /// since an element found by selector is a different handle on the same node.
    pub(crate) fn told_at(&self, element: &MountedData, host: &dyn GeometryHost) -> Option<Told> {
        self.0
            .borrow()
            .iter()
            .find(|held| host.same(&held.element, element) == SameNode::Same)
            .map(|held| held.told)
    }
}
