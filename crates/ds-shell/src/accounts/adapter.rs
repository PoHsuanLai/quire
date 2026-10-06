//! The one seam between the account sheets and the design system's components that are about to
//! be made controlled: a text field that holds a secret, a text field the host owns, a list with
//! a cursor the host owns, a pop-up button with a value the host owns, buttons that answer Return
//! and Escape, a copy to the clipboard, and the keyboard's first stop. Every other file in this
//! directory reaches those pieces through here, so changing what they are built on changes this
//! file and no other.
//!
//! What today's pieces cannot do, and how this file covers it:
//! - a secure `TextField` keeps its own text, so the host's value cannot empty it; the entry
//!   remounts it when the host hands back an empty secret after a typed one;
//! - the clipboard is the host's (`ds-shell` never touches the system), so a copy button reports
//!   the text through an event and shows the host's `CopyState`.

use super::hidden::Hidden;
use super::model::{CopyState, FieldText, PickerChoice};
use dioxus::prelude::*;
use ds::components::content::label::{Label, LabelStyle};
use ds::components::content::provider_mark::{MarkProvider, ProviderMark};
use ds::components::controls::button::Button;
use ds::components::controls::button_model::{Answers, ButtonRole};
use ds::components::fields::text_field::TextField;
use ds::components::fields::text_field_focus::FieldFocus;
use ds::components::fields::text_field_model::{FieldKind, Invalid, Validity};
use ds::components::lists::list::model::ListStyle;
use ds::components::lists::row::row::Row;
use ds::components::menus::item::item::MenuItem;
use ds::components::menus::pop_up_button::PopUpButton;
use ds::focus::soon::focus_soon;
use ds::prelude::List;
use ds::prelude::ListItem;
use ds::root::common::Common;
use ds_core::vocab::{Availability, Selection};
use ds_motion::detail::stamp::EventStamp;
use ds_style::tokens::control_size::ControlSize;
use std::hash::Hash;

/// Where the keyboard starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Landing {
    /// In this control, as it mounts.
    Here,
    /// Wherever the person puts it.
    #[default]
    Anywhere,
}

/// How a text entry is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum EntryLook {
    /// A bezeled field of a form.
    #[default]
    Field,
    /// A search field.
    Search,
}

/// A rejection to show under an entry: the words, and the attempt it answers (a new attempt
/// shakes a secret again).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Rejection {
    /// What is wrong.
    pub(crate) message: String,
    /// The submit it answers.
    pub(crate) attempt: u32,
}

/// A text entry the host owns: `text` is what it shows, `oninput` hears each change, and the
/// variant of `text` says whether the entry hides what is typed.
#[component]
pub(crate) fn Entry(
    label: String,
    text: FieldText,
    #[props(into)] placeholder: String,
    #[props(default)] look: EntryLook,
    #[props(default)] landing: Landing,
    #[props(default)] rejection: Option<Rejection>,
    #[props(default)] availability: Availability,
    oninput: EventHandler<FieldText>,
) -> Element {
    let mut seen = use_hook(|| CopyValue::new((0u32, 0usize)));
    let (mut round, typed) = *seen.peek();
    let hidden = matches!(text, FieldText::Secret(_));
    let (kind, value) = match (&text, look) {
        (FieldText::Secret(secret), _) => (FieldKind::Secure, secret.reveal().to_owned()),
        (FieldText::Plain(plain), EntryLook::Field) => (FieldKind::Plain, plain.clone()),
        (FieldText::Plain(plain), EntryLook::Search) => (FieldKind::Search, plain.clone()),
    };
    if hidden && value.is_empty() && typed > 0 {
        round += 1;
    }
    seen.set((round, value.len()));
    let validity = rejection.map_or(Validity::Valid, |rejection| {
        Validity::Invalid(Invalid {
            message: rejection.message.into(),
            stamp: EventStamp(rejection.attempt),
        })
    });
    let focus = match landing {
        Landing::Here => FieldFocus::OnMount,
        Landing::Anywhere => FieldFocus::Manual,
    };
    rsx! {
        for generation in [round] {
            TextField {
                key: "{generation}",
                label: label.clone(),
                value: value.clone(),
                placeholder: placeholder.clone(),
                kind,
                validity: validity.clone(),
                availability,
                focus,
                oninput: move |next: String| {
                    oninput.call(if hidden { FieldText::Secret(Hidden::new(next)) } else { FieldText::Plain(next) })
                },
            }
        }
    }
}

/// One row of a [`PickRows`] list.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PickRow<K> {
    /// The row's identity.
    pub(crate) key: K,
    /// What it says.
    pub(crate) title: String,
    /// The provider mark that leads it.
    pub(crate) mark: MarkProvider,
}

/// A list of rows with a mark, one of them under the cursor the host owns: the arrow keys call
/// `oncursor`, a press or Return on a row calls `onpick`.
#[component]
pub(crate) fn PickRows<K: Clone + PartialEq + Hash + 'static>(
    label: String,
    rows: Vec<PickRow<K>>,
    cursor: Option<K>,
    oncursor: EventHandler<K>,
    onpick: EventHandler<K>,
) -> Element {
    let items = rows
        .into_iter()
        .map(|row| {
            let selection = if cursor.as_ref() == Some(&row.key) {
                Selection::Selected
            } else {
                Selection::Unselected
            };
            let key = row.key.clone();
            let title = row.title.clone();
            ListItem::row(
                row.key,
                row.title.clone(),
                rsx! {
                    Row {
                        title,
                        content: Some(rsx! {
                            span { class: "ds-acc-pick",
                                ProviderMark { provider: row.mark, size: ControlSize::Regular }
                                Label { text: row.title.clone(), style: LabelStyle::Body }
                            }
                        }),
                        state: ds_core::vocab::RowState { selection, ..Default::default() },
                        onclick: move |_| onpick.call(key.clone()),
                    }
                },
            )
        })
        .collect();
    rsx! {
        List {
            label,
            items,
            style: ListStyle::Inset,
            cursor: cursor.clone(),
            onselect: move |key| oncursor.call(key),
            onpick: move |key| onpick.call(key),
        }
    }
}

/// What a button of a sheet means to its keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Intent {
    /// Return presses it; drawn in the accent.
    Default,
    /// Escape presses it.
    Cancel,
    /// Neither.
    #[default]
    Plain,
}

/// A sheet's button. `landing: Landing::Here` puts the keyboard on it as it mounts, so Return
/// and Space reach it without a click.
#[component]
pub(crate) fn Action(
    #[props(into)] label: String,
    #[props(default)] intent: Intent,
    #[props(default)] landing: Landing,
    #[props(default)] availability: Availability,
    onclick: EventHandler<()>,
) -> Element {
    let (answers, role) = match intent {
        Intent::Default => (Answers::Return, ButtonRole::Normal),
        Intent::Cancel => (Answers::Escape, ButtonRole::Normal),
        Intent::Plain => (Answers::Nothing, ButtonRole::Normal),
    };
    let common = match landing {
        Landing::Here => Common {
            mounted: Some(EventHandler::new(|event: MountedEvent| {
                focus_soon(event.data())
            })),
            ..Common::default()
        },
        Landing::Anywhere => Common::default(),
    };
    rsx! {
        Button { label, answers, role, availability, common, onclick: move |_| onclick.call(()) }
    }
}

/// A button that copies `text`: the host does the copying (it hears `oncopy`), and `state` says
/// whether it has, so the label can say "Copied".
#[component]
pub(crate) fn CopyAction(
    #[props(into)] label: String,
    #[props(into)] copied: String,
    text: String,
    state: CopyState,
    #[props(default)] intent: Intent,
    #[props(default)] landing: Landing,
    oncopy: EventHandler<String>,
) -> Element {
    let shown = match state {
        CopyState::Idle => label,
        CopyState::Copied => copied,
    };
    rsx! {
        Action { label: shown, intent, landing, onclick: move |()| oncopy.call(text.clone()) }
    }
}

/// A pop-up button over accounts and "Add Account...", its value the host's.
#[component]
pub(crate) fn ChoiceMenu(
    items: Vec<MenuItem<PickerChoice>>,
    value: Option<PickerChoice>,
    #[props(into)] title: String,
    onpick: EventHandler<PickerChoice>,
) -> Element {
    rsx! {
        PopUpButton { items, value, title: Some(title), onpick }
    }
}
