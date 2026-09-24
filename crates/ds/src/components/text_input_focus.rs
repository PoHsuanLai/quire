//! When a `TextInput` takes the caret, and the bookkeeping that serves a focus request once.

use crate::focus::host::focus_soon_told;
use crate::focus::request::{FocusRequest, FocusTicket};
use crate::focus::select::Select;
use dioxus::prelude::*;

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
    fn select(self) -> Select {
        match self {
            Focus::Controlled(request) => request.select(),
            Focus::OnMount | Focus::Manual => Select::None,
        }
    }

    /// Whether the field takes the focus as it mounts.
    pub(crate) fn on_mount(self) -> bool {
        matches!(self, Focus::OnMount | Focus::Controlled(_))
    }
}

/// The field's element and the last focus ticket it served.
#[derive(Clone, Copy)]
pub(crate) struct FieldFocus {
    element: CopyValue<Option<std::rc::Rc<MountedData>>>,
    served: CopyValue<FocusTicket>,
}

impl FieldFocus {
    /// The hook: one per field, kept across renders.
    pub(crate) fn use_new() -> Self {
        FieldFocus {
            element: use_hook(|| CopyValue::new(None)),
            served: use_hook(|| CopyValue::new(FocusTicket::default())),
        }
    }

    /// Keep the element, and take the focus if the field asks for it on mount. Focus goes
    /// through `focus_soon`, which waits out a document the renderer holds (sill Q43).
    pub(crate) fn mounted(self, focus: Focus, event: &MountedEvent, told: EventHandler<()>) {
        let mut element = self.element;
        let mut served = self.served;
        element.set(Some(event.data()));
        if let Focus::Controlled(request) = focus {
            served.set(request.peek());
        }
        if focus.on_mount() {
            focus_soon_told(event.data(), focus.select(), told);
        }
    }

    /// Serve a request made since the last one, once the element is mounted.
    pub(crate) fn follow(self, request: FocusRequest, told: EventHandler<()>) {
        let ticket = request.ticket();
        let mut served = self.served;
        let Some(element) = self.element.peek().clone() else {
            return;
        };
        if ticket != *served.peek() {
            served.set(ticket);
            focus_soon_told(element, request.select(), told);
        }
    }
}
