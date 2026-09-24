//! TextInput: the one field every text input uses (design/04-COMPONENTS.md section 6).

use crate::components::vocab::Availability;
use crate::focus::host::focus_soon;
use crate::focus::request::{FocusRequest, FocusTicket};
use dioxus::html::{Code, HasKeyboardData, Key, Location, Modifiers, ModifiersInteraction};
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

/// A key event copied out of its `Rc`, so it can be handed on by value: `KeyboardData` is not
/// `Clone`, and the `serialize` feature that offers a copy is not in the pinned set.
struct KeySnapshot {
    key: Key,
    code: Code,
    location: Location,
    modifiers: Modifiers,
    repeating: Repeat,
    composing: Composing,
}

/// Whether a key is auto-repeating.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Repeat {
    Held,
    Once,
}

/// Whether a key arrives inside an IME composition.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Composing {
    Inside,
    Outside,
}

impl ModifiersInteraction for KeySnapshot {
    fn modifiers(&self) -> Modifiers {
        self.modifiers
    }
}

impl HasKeyboardData for KeySnapshot {
    fn key(&self) -> Key {
        self.key.clone()
    }

    fn code(&self) -> Code {
        self.code
    }

    fn location(&self) -> Location {
        self.location
    }

    fn is_auto_repeating(&self) -> bool {
        self.repeating == Repeat::Held
    }

    fn is_composing(&self) -> bool {
        self.composing == Composing::Inside
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// A key event as an owned value, for an `onkey` handler.
pub(crate) fn owned_key(event: &KeyboardData) -> KeyboardData {
    KeyboardData::new(KeySnapshot {
        key: event.key(),
        code: event.code(),
        location: event.location(),
        modifiers: event.modifiers(),
        repeating: if event.is_auto_repeating() {
            Repeat::Held
        } else {
            Repeat::Once
        },
        composing: if event.is_composing() {
            Composing::Inside
        } else {
            Composing::Outside
        },
    })
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
    fn mounted(self, focus: Focus, event: &MountedEvent) {
        let mut element = self.element;
        let mut served = self.served;
        element.set(Some(event.data()));
        if let Focus::Controlled(request) = focus {
            served.set(request.peek());
        }
        if focus.on_mount() {
            focus_soon(event.data());
        }
    }

    /// Serve a request made since the last one, once the element is mounted.
    fn follow(self, request: FocusRequest) {
        let ticket = request.ticket();
        let mut served = self.served;
        let Some(element) = self.element.peek().clone() else {
            return;
        };
        if ticket != *served.peek() {
            served.set(ticket);
            focus_soon(element);
        }
    }
}

/// A single-line text field. `focus: Focus::OnMount` puts the caret in it when it mounts (and
/// writes `autofocus` for a webview); `Focus::Controlled(request)` does too, and again at each
/// `request.request()`.
#[component]
pub fn TextInput(
    variant: InputVariant,
    label: String,
    value: String,
    #[props(default)] placeholder: String,
    #[props(default)] availability: Availability,
    oninput: EventHandler<String>,
    #[props(default)] onkey: EventHandler<KeyboardData>,
    #[props(default)] focus: Focus,
) -> Element {
    let field = FieldFocus {
        element: use_hook(|| CopyValue::new(None)),
        served: use_hook(|| CopyValue::new(FocusTicket::default())),
    };
    if let Focus::Controlled(request) = focus {
        field.follow(request);
    }
    let shown = placeholder_shown(&value, &placeholder).map(str::to_string);
    let aria_placeholder = (!placeholder.is_empty()).then_some(placeholder.clone());
    rsx! {
        span { class: "ds-input-wrap", "data-variant": variant.slug(),
            input {
                class: "ds-input",
                "data-variant": variant.slug(),
                r#type: "text",
                "aria-label": "{label}",
                "aria-placeholder": aria_placeholder,
                "aria-disabled": availability.aria_disabled(),
                autocomplete: "off",
                autofocus: focus.on_mount().then_some("true"),
                value: "{value}",
                onmounted: move |event| field.mounted(focus, &event),
                oninput: move |event| {
                    if availability == Availability::Enabled {
                        oninput.call(event.value());
                    }
                },
                onkeydown: move |event| onkey.call(owned_key(&event.data())),
            }
            if let Some(text) = shown {
                span { class: "ds-input-placeholder", "aria-hidden": "true", "{text}" }
            }
        }
    }
}
