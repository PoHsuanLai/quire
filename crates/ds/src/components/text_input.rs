//! TextInput: the one field every text input uses (design/04-COMPONENTS.md section 6).

use crate::components::text_input_parts::{Field, Handlers, area, file, line};
use crate::components::vocab::Availability;
use dioxus::prelude::*;

use crate::components::text_input_focus::FieldFocus;
pub use crate::components::text_input_focus::Focus;
pub use crate::components::text_input_kind::{Grow, Rows, TextInputKind};
use crate::focus::FieldHandle;
use crate::focus::targets::Told;

/// The field's face: Boxed for a standalone field, Inline inside another container, Bare in
/// the text it edits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputVariant {
    /// `.inp`: bordered, its own focus ring.
    Boxed,
    /// `.inp.inline`: transparent; the container shows focus.
    Inline,
    /// No box, no padding and no face of its own (mailo gaps 4): font, size, weight, tracking
    /// and colour are the parent's, so a property row's value or a title is edited where it
    /// reads. Only the caret (`--accent`) and the selection are styled.
    Bare,
}

/// The name mailo gaps 4 asked for: a field's face is its variant, so `variant:
/// FieldFace::Bare` reads as it means. One type, so a face and a variant cannot disagree.
pub type FieldFace = InputVariant;

impl InputVariant {
    /// The `data-variant` word.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            InputVariant::Boxed => "boxed",
            InputVariant::Inline => "inline",
            InputVariant::Bare => "bare",
        }
    }
}

/// The text a masked or plain field shows: a `Secret`'s own state, else the caller's `value`.
fn held(kind: TextInputKind, value: String, typed: Signal<String>) -> String {
    match kind {
        TextInputKind::Secret => typed(),
        _ => value,
    }
}

/// A text field. `focus: Focus::OnMount` puts the caret in it when it mounts (and writes
/// `autofocus` for a webview); `Focus::Controlled(request)` does too, and again at each
/// `request.request()`. `onkey` hears each key as the event itself, so a caller that takes a
/// key can `prevent_default` it (sill FINDINGS Q61).
///
/// `onfocus` and `onblur` hear the caret arrive and leave, so a caller can tell "the person is
/// typing in a field" from "a key for the window". They fire for a click or Tab (the renderer's
/// own events) and for the focus seam: when `Focus::OnMount` or `Focus::Controlled` puts the
/// caret in the field through a host that dispatches no event (Blitz), the field calls `onfocus`
/// itself. A range is [`Slider`](crate::Slider).
///
/// `handle` ([`use_field_handle`](crate::use_field_handle)) is the caller's grip on the field
/// from any handler: `focus(Select)`, `blur()` and the mounted element. A focus or blur through
/// it (or through [`focus_by_selector`](crate::focus_by_selector)) calls `onfocus`/`onblur` once
/// on Blitz too.
///
/// `kind` picks what it holds ([`TextInputKind`]): `Secret` keeps its text out of the markup,
/// `File` asks the host through `on_pick`, `Multiline` is a `textarea`. `onchange` hears the
/// value committed: Enter in a one-line field, or the caret leaving any field (Blitz sends no
/// `change` event, so the field makes its own, the same on both renderers).
#[component]
pub fn TextInput(
    variant: InputVariant,
    label: String,
    value: String,
    #[props(default)] placeholder: String,
    #[props(default)] availability: Availability,
    oninput: EventHandler<String>,
    #[props(default)] onkey: EventHandler<KeyboardEvent>,
    #[props(default)] focus: Focus,
    #[props(default)] kind: TextInputKind,
    #[props(default)] onfocus: EventHandler<()>,
    #[props(default)] onblur: EventHandler<()>,
    #[props(default)] onchange: EventHandler<String>,
    #[props(default)] on_pick: EventHandler<()>,
    #[props(default)] handle: Option<FieldHandle>,
) -> Element {
    let focuser = FieldFocus::use_new(handle);
    let mut typed = use_signal(String::new);
    if let Focus::Controlled(request) = focus {
        focuser.follow(request, onfocus);
    }
    let text = held(kind, value, typed);
    let committed = text.clone();
    let blurred = text.clone();
    let told = Told {
        focus: use_callback(move |()| onfocus.call(())),
        blur: use_callback(move |()| {
            onchange.call(blurred.clone());
            onblur.call(());
        }),
    };
    let field = Field {
        variant,
        label,
        placeholder,
        availability,
        focus,
        focuser,
        handlers: Handlers {
            oninput: EventHandler::new(move |next: String| {
                typed.set(next.clone());
                oninput.call(next);
            }),
            onkey,
            onfocus,
            onblur,
            onchange: EventHandler::new(move |()| onchange.call(committed.clone())),
        },
        told,
    };
    match kind {
        TextInputKind::File => file(field, text, on_pick),
        TextInputKind::Multiline { rows, grow } => area(field, grow.rows(rows, &text), text),
        TextInputKind::Text | TextInputKind::Password | TextInputKind::Secret => {
            line(field, kind, text)
        }
    }
}
