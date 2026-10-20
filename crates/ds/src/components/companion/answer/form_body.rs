//! A compact form: the title and a labelled input per missing value. Each edit is reported under
//! the field's key.

use crate::components::companion::form::model::{
    FieldKeyId, FieldNeed, FormField, FormInput, FormValue,
};
use crate::components::controls::chip::{Chip, ChipVariant};
use crate::components::fields::text_field::TextField;
use crate::components::fields::text_field_model::{FieldKind, FieldRows};
use dioxus::prelude::*;
use ds_core::vocab::Check;
use ds_core::word::Word;

/// The text a field holds, for an input that types.
fn typed(value: &FormValue) -> String {
    match value {
        FormValue::Text(text) | FormValue::Number(text) => text.clone(),
        FormValue::Empty | FormValue::Choice(_) | FormValue::Entity(_) => String::new(),
    }
}

/// The key a field holds, for an input that picks.
fn picked(value: &FormValue) -> Option<&str> {
    match value {
        FormValue::Choice(key) | FormValue::Entity(key) => Some(key),
        FormValue::Empty | FormValue::Text(_) | FormValue::Number(_) => None,
    }
}

/// How the input's typed text becomes a value.
fn typed_value(input: &FormInput, text: String) -> FormValue {
    match input {
        FormInput::Number => FormValue::Number(text),
        _ => FormValue::Text(text),
    }
}

/// The form as the card body.
#[component]
pub fn FormBody(
    title: String,
    fields: Vec<FormField>,
    on_edit: EventHandler<(FieldKeyId, FormValue)>,
) -> Element {
    rsx! {
        div { class: "ds-card-form",
            div { class: "ds-card-form-title", "{title}" }
            for field in fields {
                FieldView { key: "{field.key.0}", field, on_edit }
            }
        }
    }
}

#[component]
fn FieldView(field: FormField, on_edit: EventHandler<(FieldKeyId, FormValue)>) -> Element {
    let need = field.need.slug();
    rsx! {
        div { class: "ds-card-form-field", "data-need": need,
            span { class: "ds-card-label",
                "{field.label}"
                if field.need == FieldNeed::Optional {
                    span { class: "ds-card-optional", " (optional)" }
                }
            }
            FieldInput { field, on_edit }
        }
    }
}

/// What a picking input picks, and so the value it reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PickKind {
    /// One of the field's own choices.
    Choice,
    /// A thing from an app.
    Entity,
}

impl PickKind {
    fn value(self, key: String) -> FormValue {
        match self {
            PickKind::Choice => FormValue::Choice(key),
            PickKind::Entity => FormValue::Entity(key),
        }
    }
}

#[component]
fn FieldInput(field: FormField, on_edit: EventHandler<(FieldKeyId, FormValue)>) -> Element {
    let key = field.key.clone();
    match field.input.clone() {
        FormInput::Choice(choices) => rsx! {
            Picks { field, choices, kind: PickKind::Choice, on_edit }
        },
        FormInput::Entity { suggestions } => {
            let choices = suggestions
                .into_iter()
                .map(|option| (option.key, option.title))
                .collect();
            rsx! {
                Picks { field, choices, kind: PickKind::Entity, on_edit }
            }
        }
        input => {
            let (kind, rows) = match &input {
                FormInput::Lines => (FieldKind::Multiline, FieldRows::Three),
                _ => (FieldKind::Plain, FieldRows::default()),
            };
            rsx! {
                TextField {
                    label: field.label.clone(),
                    value: typed(&field.value),
                    kind,
                    rows,
                    validity: field.validity.clone(),
                    oninput: move |text| on_edit.call((key.clone(), typed_value(&input, text))),
                }
            }
        }
    }
}

/// One pressable chip per choice, the picked one pressed in.
#[component]
fn Picks(
    field: FormField,
    choices: Vec<(String, String)>,
    kind: PickKind,
    on_edit: EventHandler<(FieldKeyId, FormValue)>,
) -> Element {
    let current = picked(&field.value).map(str::to_owned);
    rsx! {
        div { class: "ds-card-form-picks", role: "group", "aria-label": "{field.label}",
            for (key , label) in choices {
                Chip {
                    key: "{key}",
                    variant: ChipVariant::Neutral,
                    text: label,
                    pressed: Some(pressed(current.as_deref(), &key)),
                    onpress: {
                        let field_key = field.key.clone();
                        move |_| on_edit.call((field_key.clone(), kind.value(key.clone())))
                    },
                }
            }
        }
    }
}

/// Whether `key` is the pick, as the toggle state it draws.
fn pressed(picked: Option<&str>, key: &str) -> Check {
    match picked == Some(key) {
        true => Check::On,
        false => Check::Off,
    }
}
