//! TextInput: the one field every text input uses (design/04-COMPONENTS.md section 6).

use crate::components::vocab::Availability;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Focus {
    /// As soon as it is mounted: the palette's input, the bubble's link field.
    OnMount,
    /// Only when the user or the consumer puts it there.
    #[default]
    Manual,
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

/// Take focus now if the field asks for it on mount. Focus is best-effort: a renderer without
/// it still shows the field, and the user can click into it.
fn focus_on_mount(focus: Focus, event: &MountedEvent) {
    if focus == Focus::OnMount {
        let mounted = event.data();
        spawn(async move {
            let _ = mounted.set_focus(true).await;
        });
    }
}

/// A single-line text field. `focus: Focus::OnMount` puts the caret in it when it mounts (and
/// writes `autofocus` for a webview).
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
                autofocus: (focus == Focus::OnMount).then_some("true"),
                value: "{value}",
                onmounted: move |event| focus_on_mount(focus, &event),
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
