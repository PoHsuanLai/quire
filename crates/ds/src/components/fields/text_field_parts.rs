//! The one-line `input` a `TextField` draws: the input itself, the placeholder over it and, for a
//! secure field, the dots and the caret that replace what the renderer would draw.

use crate::components::fields::text_field_area::area;
use crate::components::fields::text_field_focus::{FieldFocus, FieldFocuser};
use crate::components::fields::text_field_mask::{CaretMark, MaskCaret, MaskParts};
use crate::components::fields::text_field_model::{FieldBezel, FieldKind, FieldRows};
use crate::focus::targets::Told;
use dioxus::prelude::*;
use ds_core::vocab::Availability;
use ds_core::word::Word;
use ds_style::tokens::control_size::ControlSize;

/// Where a field's events go. `onchange` takes no value: the field already knows it.
#[derive(Clone, Copy)]
pub(crate) struct Handlers {
    pub oninput: EventHandler<String>,
    pub onkey: EventHandler<KeyboardEvent>,
    pub onfocus: EventHandler<()>,
    pub onblur: EventHandler<()>,
    pub onchange: EventHandler<()>,
}

/// Everything the line needs besides its value.
pub(crate) struct Field {
    pub bezel: FieldBezel,
    /// How tall a multi-line field is; a line ignores it.
    pub rows: FieldRows,
    pub size: ControlSize,
    pub label: String,
    /// The input's own `id`, from the caller's `Common`.
    pub id: Option<String>,
    pub placeholder: String,
    pub availability: Availability,
    /// `aria-invalid`, while the value is rejected.
    pub invalid: Option<&'static str>,
    pub focus: FieldFocus,
    pub focuser: FieldFocuser,
    /// A masked field's own caret over its dots.
    pub caret: MaskCaret,
    pub handlers: Handlers,
    /// What a host's focus write tells the field, which no renderer event will.
    pub told: Told,
}

/// The placeholder as a span over the field, shown while the value is empty: Blitz draws no
/// `placeholder` attribute and has no `::placeholder` (O-23's fallback). The input carries the
/// text as `aria-placeholder` instead, so a browser does not draw it twice.
pub(crate) fn placeholder_shown(value: &str, placeholder: &str) -> Option<String> {
    (value.is_empty() && !placeholder.is_empty()).then(|| placeholder.to_string())
}

/// `aria-placeholder`, when there is a placeholder at all.
pub(crate) fn aria_placeholder(placeholder: &str) -> Option<String> {
    (!placeholder.is_empty()).then(|| placeholder.to_string())
}

/// The field's text as it draws: a line (text, a search or a secret; a secure field writes no
/// `value`) or, for a multi-line field, a `textarea`.
pub(crate) fn line(field: Field, kind: FieldKind, value: String) -> Element {
    if kind == FieldKind::Multiline {
        return area(field, value);
    }
    let Field {
        bezel,
        rows: _,
        size,
        label,
        id,
        placeholder,
        availability,
        invalid,
        focus,
        focuser,
        caret,
        handlers,
        told,
    } = field;
    let count = kind.mask(&value).map_or(0, |dots| dots.chars().count());
    let owner = caret.owner(count);
    let mask = (count > 0).then(|| MaskParts::cut(count, caret.selection(), owner));
    let shown = placeholder_shown(&value, &placeholder);
    let written = (kind != FieldKind::Secure).then_some(value);
    rsx! {
        span {
            class: "ds-input-wrap",
            "data-variant": bezel.slug(),
            "data-size": size.slug(),
            input {
                id,
                class: "ds-input",
                "data-variant": bezel.slug(),
                r#type: kind.input_type(),
                "data-kind": kind.data_kind(),
                "data-caret": owner.data_caret(),
                "aria-label": "{label}",
                "aria-placeholder": aria_placeholder(&placeholder),
                "aria-invalid": invalid,
                "aria-disabled": availability.aria_disabled(),
                "aria-busy": availability.aria_busy(),
                autocomplete: "off",
                autofocus: focus.on_mount().then_some("true"),
                value: written,
                onmounted: move |event| {
                    caret.mounted(&event);
                    focuser.mounted(focus, &event, told);
                },
                onfocus: move |_| {
                    caret.refresh();
                    handlers.onfocus.call(());
                },
                onblur: move |_| {
                    caret.refresh();
                    handlers.onchange.call(());
                    handlers.onblur.call(());
                },
                onmousedown: move |_| caret.refresh(),
                onmouseup: move |_| caret.refresh(),
                oninput: move |event| {
                    caret.refresh();
                    if availability == Availability::Enabled {
                        handlers.oninput.call(event.value());
                    }
                },
                onkeydown: move |event: KeyboardEvent| {
                    caret.refresh();
                    if event.key() == Key::Enter {
                        handlers.onchange.call(());
                    }
                    handlers.onkey.call(event);
                },
            }
            if let Some(parts) = mask {
                span { class: "ds-input-mask", "aria-hidden": "true",
                    "{parts.before}"
                    if !parts.selected.is_empty() {
                        span { class: "ds-input-mask-selected", "{parts.selected}" }
                    }
                    if parts.caret == CaretMark::Shown {
                        span { class: "ds-input-caret" }
                    }
                    "{parts.after}"
                }
            }
            if let Some(text) = shown {
                span { class: "ds-input-placeholder", "aria-hidden": "true", "{text}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{aria_placeholder, placeholder_shown};

    #[test]
    fn the_placeholder_shows_only_over_an_empty_value() {
        const CASES: &[(&str, &str, Option<&str>)] = &[
            ("", "Add a person", Some("Add a person")),
            ("Dana", "Add a person", None),
            ("", "", None),
        ];
        for &(value, placeholder, want) in CASES {
            assert_eq!(placeholder_shown(value, placeholder).as_deref(), want);
        }
        assert_eq!(aria_placeholder("").as_deref(), None);
        assert_eq!(aria_placeholder("x").as_deref(), Some("x"));
    }
}
