//! When a `TextInput` takes the caret, and the bookkeeping that serves a focus request once.

use crate::focus::field::FieldHandle;
use crate::focus::host::focus_soon_told;
use crate::focus::request::{FocusRequest, FocusTicket};
use crate::focus::select::Landing;
use crate::focus::targets::{FocusTarget, FocusTargets, Told};
use dioxus::prelude::*;
use std::rc::Rc;

/// When a field takes keyboard focus (design/06-INTERACTIONS.md section 17).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Focus {
    /// As soon as it is mounted: the palette's input, the bubble's link field.
    OnMount,
    /// Only when the user or the consumer puts it there.
    #[default]
    Manual,
    /// As it mounts, and again each time the caller calls [`FocusRequest::request`]: a menu
    /// that took the keyboard hands it back to the field when it closes (sill FINDINGS Q44).
    Controlled(FocusRequest),
}

impl Focus {
    /// What the field does with its text when the focus lands: a controlled request's choice.
    fn landing(self) -> Landing {
        match self {
            Focus::Controlled(request) => request.landing(),
            Focus::OnMount | Focus::Manual => Landing::Leave,
        }
    }

    /// Whether the field takes the focus as it mounts.
    pub(crate) fn on_mount(self) -> bool {
        matches!(self, Focus::OnMount | Focus::Controlled(_))
    }
}

/// The field's element, the last focus ticket it served, and where a focus from outside finds
/// it (its caller's [`FieldHandle`] and the document's [`FocusTargets`]).
#[derive(Clone)]
pub(crate) struct FieldFocus {
    element: CopyValue<Option<Rc<MountedData>>>,
    served: CopyValue<FocusTicket>,
    handle: Option<FieldHandle>,
    targets: FocusTargets,
    owner: ScopeId,
}

impl FieldFocus {
    /// The hook: one per field, kept across renders; the field leaves the document's targets as
    /// it unmounts.
    pub(crate) fn use_new(handle: Option<FieldHandle>) -> Self {
        let targets = use_hook(FocusTargets::root);
        let owner = use_hook(dioxus::core::current_scope_id);
        let leaving = targets.clone();
        use_drop(move || leaving.leave(owner));
        FieldFocus {
            element: use_hook(|| CopyValue::new(None)),
            served: use_hook(|| CopyValue::new(FocusTicket::default())),
            handle,
            targets,
            owner,
        }
    }

    /// Keep the element, hand it to the caller's handle and the document's targets, and take
    /// the focus if the field asks for it on mount. Focus goes through `focus_soon`, which waits
    /// out a document the renderer holds (sill Q43).
    pub(crate) fn mounted(&self, focus: Focus, event: &MountedEvent, told: Told) {
        let mut element = self.element;
        let mut served = self.served;
        element.set(Some(event.data()));
        let target = FocusTarget {
            owner: self.owner,
            element: event.data(),
            told,
        };
        if let Some(handle) = self.handle {
            handle.fill(target.clone());
        }
        self.targets.enter(target);
        if let Focus::Controlled(request) = focus {
            served.set(request.peek());
        }
        if focus.on_mount() {
            focus_soon_told(event.data(), focus.landing(), told.focus);
        }
    }

    /// Serve a request made since the last one, once the element is mounted.
    pub(crate) fn follow(&self, request: FocusRequest, told: EventHandler<()>) {
        let ticket = request.ticket();
        let mut served = self.served;
        let Some(element) = self.element.peek().clone() else {
            return;
        };
        if ticket != *served.peek() {
            served.set(ticket);
            focus_soon_told(element, request.landing(), told);
        }
    }
}
