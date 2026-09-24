//! The three shapes a `TextInput` draws: one line (text, password, secret), a `textarea`, and
//! a file's name beside its "Choose…" button.

use crate::components::icon_button::{IconButton, IconButtonVariant};
use crate::components::text_input::InputVariant;
use crate::components::text_input_focus::{FieldFocus, Focus};
use crate::components::text_input_kind::{Rows, TextInputKind};
use crate::components::vocab::Availability;
use crate::icon::Icon;
use dioxus::prelude::*;

/// Where a field's events go. `onchange` takes no value: the field already knows it.
#[derive(Clone, Copy)]
pub(crate) struct Handlers {
    pub oninput: EventHandler<String>,
    pub onkey: EventHandler<KeyboardEvent>,
    pub onfocus: EventHandler<()>,
    pub onblur: EventHandler<()>,
    pub onchange: EventHandler<()>,
}

/// Everything a shape needs besides its value.
pub(crate) struct Field {
    pub variant: InputVariant,
    pub label: String,
    pub placeholder: String,
    pub availability: Availability,
    pub focus: Focus,
    pub focuser: FieldFocus,
    pub handlers: Handlers,
}

/// The placeholder as a span over the field, shown while the value is empty: Blitz draws no
/// `placeholder` attribute and has no `::placeholder` (O-23's fallback). The input carries the
/// text as `aria-placeholder` instead, so a browser does not draw it twice.
fn placeholder_shown(value: &str, placeholder: &str) -> Option<String> {
    (value.is_empty() && !placeholder.is_empty()).then(|| placeholder.to_string())
}

/// `aria-placeholder`, when there is a placeholder at all.
fn aria_placeholder(placeholder: &str) -> Option<String> {
    (!placeholder.is_empty()).then(|| placeholder.to_string())
}

/// A single line: text, a password, or a secret. A secret writes no `value`.
pub(crate) fn line(field: Field, kind: TextInputKind, value: String) -> Element {
    let Field {
        variant,
        label,
        placeholder,
        availability,
        focus,
        focuser,
        handlers,
    } = field;
    let mask = kind.mask(&value);
    let shown = placeholder_shown(&value, &placeholder);
    let written = (kind != TextInputKind::Secret).then_some(value);
    rsx! {
        span { class: "ds-input-wrap", "data-variant": variant.slug(),
            input {
                class: "ds-input",
                "data-variant": variant.slug(),
                r#type: kind.input_type(),
                "data-kind": kind.data_kind(),
                "aria-label": "{label}",
                "aria-placeholder": aria_placeholder(&placeholder),
                "aria-disabled": availability.aria_disabled(),
                autocomplete: "off",
                autofocus: focus.on_mount().then_some("true"),
                value: written,
                onmounted: move |event| focuser.mounted(focus, &event, handlers.onfocus),
                onfocus: move |_| handlers.onfocus.call(()),
                onblur: move |_| {
                    handlers.onchange.call(());
                    handlers.onblur.call(());
                },
                oninput: move |event| {
                    if availability == Availability::Enabled {
                        handlers.oninput.call(event.value());
                    }
                },
                onkeydown: move |event: KeyboardEvent| {
                    if event.key() == Key::Enter {
                        handlers.onchange.call(());
                    }
                    handlers.onkey.call(event);
                },
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

/// A `textarea` drawn at `rows` lines (already grown to its content when it grows). Blitz
/// reads its text from the `value` attribute, as it does an input's, and sizes it by `rows`.
pub(crate) fn area(field: Field, rows: Rows, value: String) -> Element {
    let Field {
        variant,
        label,
        placeholder,
        availability,
        focus,
        focuser,
        handlers,
    } = field;
    let shown = placeholder_shown(&value, &placeholder);
    rsx! {
        span { class: "ds-input-wrap", "data-variant": variant.slug(), "data-kind": "multiline",
            textarea {
                class: "ds-input",
                "data-variant": variant.slug(),
                "data-kind": "multiline",
                rows: "{rows.0}",
                "aria-label": "{label}",
                "aria-placeholder": aria_placeholder(&placeholder),
                "aria-disabled": availability.aria_disabled(),
                autofocus: focus.on_mount().then_some("true"),
                value: "{value}",
                onmounted: move |event| focuser.mounted(focus, &event, handlers.onfocus),
                onfocus: move |_| handlers.onfocus.call(()),
                onblur: move |_| {
                    handlers.onchange.call(());
                    handlers.onblur.call(());
                },
                oninput: move |event| {
                    if availability == Availability::Enabled {
                        handlers.oninput.call(event.value());
                    }
                },
                onkeydown: move |event| handlers.onkey.call(event),
            }
            if let Some(text) = shown {
                span { class: "ds-input-placeholder", "aria-hidden": "true", "{text}" }
            }
        }
    }
}

/// A file's name, read-only, and the button that asks the host to choose one. A click on the
/// name asks too. The name is the caller's `value`; the placeholder stands in while it is empty.
pub(crate) fn file(field: Field, value: String, on_pick: EventHandler<()>) -> Element {
    let Field {
        variant,
        label,
        placeholder,
        availability,
        ..
    } = field;
    let live = availability == Availability::Enabled;
    let shown = placeholder_shown(&value, &placeholder);
    let pick = move || {
        if live {
            on_pick.call(());
        }
    };
    rsx! {
        span { class: "ds-input-wrap", "data-variant": variant.slug(), "data-kind": "file",
            span {
                class: "ds-input",
                "data-variant": variant.slug(),
                "data-kind": "file",
                role: "textbox",
                "aria-readonly": "true",
                "aria-label": "{label}",
                "aria-disabled": availability.aria_disabled(),
                onclick: move |_| pick(),
                "{value}"
            }
            if let Some(text) = shown {
                span { class: "ds-input-placeholder", "aria-hidden": "true", "{text}" }
            }
            IconButton {
                variant: IconButtonVariant::Tool,
                icon: Icon::Folder,
                label: "{label}: Choose…",
                tooltip: "Choose…".to_string(),
                availability,
                onclick: move |_| pick(),
            }
        }
    }
}
