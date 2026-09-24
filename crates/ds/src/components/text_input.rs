//! TextInput: the one field every text input uses (design/04-COMPONENTS.md section 6).

use crate::components::vocab::Availability;
use crate::focus::host::focus_soon_told;
use crate::focus::request::{FocusRequest, FocusTicket};
use dioxus::prelude::*;

/// Boxed for a standalone field, Inline inside another container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputVariant {
    /// `.inp`: bordered, its own focus ring.
    Boxed,
    /// `.inp.inline`: transparent; the container shows focus.
    Inline,
}

impl InputVariant {
    /// The `data-variant` word.
    fn slug(self) -> &'static str {
        match self {
            InputVariant::Boxed => "boxed",
            InputVariant::Inline => "inline",
        }
    }
}

/// What the field holds: plain text, or a secret drawn as dots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextInputKind {
    /// `type="text"`.
    #[default]
    Text,
    /// `type="password"`. Blitz lays a password field out as text and draws its characters as
    /// typed, so the field paints its text transparent and lays a row of dots over it, one per
    /// character; a browser, which masks on its own, draws the same dots.
    Password,
}

impl TextInputKind {
    /// The `type` attribute.
    fn input_type(self) -> &'static str {
        match self {
            TextInputKind::Text => "text",
            TextInputKind::Password => "password",
        }
    }

    /// `data-kind`: written only for a password, so a text field's markup is as it was.
    fn data_kind(self) -> Option<&'static str> {
        match self {
            TextInputKind::Text => None,
            TextInputKind::Password => Some("password"),
        }
    }

    /// The dots drawn over a password's value; nothing for text or an empty value.
    fn mask(self, value: &str) -> Option<String> {
        match self {
            TextInputKind::Password if !value.is_empty() => {
                Some(value.chars().map(|_| MASK_DOT).collect())
            }
            TextInputKind::Text | TextInputKind::Password => None,
        }
    }
}

/// One masked character.
const MASK_DOT: char = '\u{2022}';

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
    /// Whether the field takes the focus as it mounts.
    fn on_mount(self) -> bool {
        matches!(self, Focus::OnMount | Focus::Controlled(_))
    }
}

/// The placeholder as a span over the field, shown while the value is empty: Blitz draws no
/// `placeholder` attribute and has no `::placeholder` (O-23's fallback). The input carries the
/// text as `aria-placeholder` instead, so a browser does not draw it twice.
fn placeholder_shown<'a>(value: &str, placeholder: &'a str) -> Option<&'a str> {
    (value.is_empty() && !placeholder.is_empty()).then_some(placeholder)
}

/// The field's element and the last focus ticket it served.
#[derive(Clone, Copy)]
struct FieldFocus {
    element: CopyValue<Option<std::rc::Rc<MountedData>>>,
    served: CopyValue<FocusTicket>,
}

impl FieldFocus {
    /// Keep the element, and take the focus if the field asks for it on mount. Focus goes
    /// through `focus_soon`, which waits out a document the renderer holds (sill Q43).
    fn mounted(self, focus: Focus, event: &MountedEvent, told: EventHandler<()>) {
        let mut element = self.element;
        let mut served = self.served;
        element.set(Some(event.data()));
        if let Focus::Controlled(request) = focus {
            served.set(request.peek());
        }
        if focus.on_mount() {
            focus_soon_told(event.data(), told);
        }
    }

    /// Serve a request made since the last one, once the element is mounted.
    fn follow(self, request: FocusRequest, told: EventHandler<()>) {
        let ticket = request.ticket();
        let mut served = self.served;
        let Some(element) = self.element.peek().clone() else {
            return;
        };
        if ticket != *served.peek() {
            served.set(ticket);
            focus_soon_told(element, told);
        }
    }
}

/// A single-line text field. `focus: Focus::OnMount` puts the caret in it when it mounts (and
/// writes `autofocus` for a webview); `Focus::Controlled(request)` does too, and again at each
/// `request.request()`. `onkey` hears each key as the event itself, so a caller that takes a
/// key can `prevent_default` it (sill FINDINGS Q61).
///
/// `onfocus` and `onblur` hear the caret arrive and leave, so a caller can tell "the person is
/// typing in a field" from "a key for the window". They fire for a click or Tab (the renderer's
/// own events) and for the focus seam: when `Focus::OnMount` or `Focus::Controlled` puts the
/// caret in the field through a host that dispatches no event (Blitz), the field calls `onfocus`
/// itself. `kind` is `Text` or `Password`; a range is [`Slider`](crate::Slider).
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
) -> Element {
    let field = FieldFocus {
        element: use_hook(|| CopyValue::new(None)),
        served: use_hook(|| CopyValue::new(FocusTicket::default())),
    };
    if let Focus::Controlled(request) = focus {
        field.follow(request, onfocus);
    }
    let mask = kind.mask(&value);
    let shown = placeholder_shown(&value, &placeholder).map(str::to_string);
    let aria_placeholder = (!placeholder.is_empty()).then_some(placeholder.clone());
    rsx! {
        span { class: "ds-input-wrap", "data-variant": variant.slug(),
            input {
                class: "ds-input",
                "data-variant": variant.slug(),
                r#type: kind.input_type(),
                "data-kind": kind.data_kind(),
                "aria-label": "{label}",
                "aria-placeholder": aria_placeholder,
                "aria-disabled": availability.aria_disabled(),
                autocomplete: "off",
                autofocus: focus.on_mount().then_some("true"),
                value: "{value}",
                onmounted: move |event| field.mounted(focus, &event, onfocus),
                onfocus: move |_| onfocus.call(()),
                onblur: move |_| onblur.call(()),
                oninput: move |event| {
                    if availability == Availability::Enabled {
                        oninput.call(event.value());
                    }
                },
                onkeydown: move |event| onkey.call(event),
            }
            if let Some(dots) = mask {
                span { class: "ds-input-mask", "aria-hidden": "true", "{dots}" }
            }
            if let Some(text) = shown {
                span { class: "ds-input-placeholder", "aria-hidden": "true", "{text}" }
            }
        }
    }
}
