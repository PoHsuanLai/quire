//! A compact form: the parameters an action still needs.

use crate::components::companion::answer::action::CardAction;
use crate::components::companion::answer::footer::CardFooter;
use crate::components::content::icon_source::IconSource;
use crate::components::fields::text_field_model::Validity;
use ds_core::word::Word;

/// A form field's key: the parameter's name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldKeyId(pub String);

/// Whether a field must be filled, `data-need`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum FieldNeed {
    /// The action cannot run without it.
    Required,
    /// It may stay empty.
    Optional,
}

/// A thing the person can pick for an entity field.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityOption {
    /// The app's key for it.
    pub key: String,
    /// Its title.
    pub title: String,
    /// Under the title.
    pub subtitle: String,
    /// Its icon.
    pub icon: IconSource,
}

/// What kind of input a field takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormInput {
    /// One line.
    Text,
    /// Several lines.
    Lines,
    /// A number.
    Number,
    /// A date.
    Date,
    /// A time.
    Time,
    /// One of these: key and label.
    Choice(Vec<(String, String)>),
    /// A thing from an app, with suggestions.
    Entity {
        /// What the app suggested.
        suggestions: Vec<EntityOption>,
    },
}

/// What a field holds.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FormValue {
    /// Nothing yet.
    #[default]
    Empty,
    /// Text.
    Text(String),
    /// A number as typed.
    Number(String),
    /// A chosen key.
    Choice(String),
    /// A chosen thing's key.
    Entity(String),
}

/// One field of the form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormField {
    /// Which parameter.
    pub key: FieldKeyId,
    /// Its label.
    pub label: String,
    /// What it takes.
    pub input: FormInput,
    /// Whether it must be filled.
    pub need: FieldNeed,
    /// What it holds.
    pub value: FormValue,
    /// Whether the value is acceptable.
    pub validity: Validity,
}

/// A form for the parameters an action still needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactForm {
    /// What the form is for.
    pub title: String,
    /// Its fields.
    pub fields: Vec<FormField>,
    /// The button that submits it.
    pub submit: CardAction,
    /// The footer every card has.
    pub footer: CardFooter,
}
