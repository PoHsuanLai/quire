//! The one seam between the account sheets and the design system's controlled components: a text
//! entry the host owns (a secret too), a list with a cursor the host owns, a pop-up button with a
//! value the host owns, buttons that answer Return and Escape, a copy to the clipboard, and the
//! keyboard's first stop. Every other file in this directory reaches those pieces through here.
//!
//! Return and Escape are the sheet's (`Sheet { on_return, onclose }`): a field's Enter bubbles to
//! it, so an entry wires no `onsubmit` of its own and the step's default action runs once.

use super::hidden::Hidden;
use super::model::{Choice, CopyState, FieldText, PickerChoice};
use dioxus::prelude::*;
use ds::components::content::avatar::AvatarSize;
use ds::components::content::icon_source::{ExternalIcon, IconSource};
use ds::components::content::icon_view::IconView;
use ds::components::content::mark_face::MarkFace;
use ds::components::content::provider_mark::{MarkProvider, MarkStyle};
use ds::components::content::text_runs::TextLine;
use ds::components::controls::button::Button;
use ds::components::controls::button_model::{Answers, ButtonFocus, ButtonRole};
use ds::components::fields::text_field::TextField;
use ds::components::fields::text_field_focus::FieldFocus;
use ds::components::fields::text_field_model::{FieldKind, FieldText as HeldBy, Invalid, Validity};
use ds::components::lists::list::model::ListStyle;
use ds::components::lists::row::size::RowSize;
use ds::components::menus::item::item::MenuItem;
use ds::components::menus::pop_up_button::PopUpButton;
use ds::prelude::ListItem;
use ds::prelude::{List, Row, RowLeading};
use ds_core::vocab::{Availability, Selection};
use ds_motion::detail::stamp::EventStamp;
use ds_style::icon::render::{IconPx, IconSize};
use ds_style::icon::url::IconUrl;
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
/// variant of `text` says whether the entry hides what is typed. A secret is the caller's too
/// (`HeldBy::Caller`): the field keeps no copy, so a host that empties or restores it is obeyed.
#[component]
pub(crate) fn Entry(
    label: String,
    text: FieldText,
    #[props(into)] placeholder: String,
    #[props(default)] look: EntryLook,
    #[props(default)] landing: Landing,
    #[props(default)] rejection: Option<Rejection>,
    #[props(default)] help: Option<String>,
    #[props(default)] availability: Availability,
    oninput: EventHandler<FieldText>,
    #[props(default)] onkey: EventHandler<KeyboardEvent>,
) -> Element {
    let hidden = matches!(text, FieldText::Secret(_));
    let (kind, held, value) = match (&text, look) {
        (FieldText::Secret(secret), _) => (
            FieldKind::Secure,
            HeldBy::Caller,
            secret.reveal().to_owned(),
        ),
        (FieldText::Plain(plain), EntryLook::Field) => {
            (FieldKind::Plain, HeldBy::Own, plain.clone())
        }
        (FieldText::Plain(plain), EntryLook::Search) => {
            (FieldKind::Search, HeldBy::Own, plain.clone())
        }
    };
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
        TextField {
            label,
            value,
            placeholder,
            kind,
            text: held,
            validity,
            help: help.map(TextLine::Plain),
            availability,
            focus,
            onkey,
            oninput: move |next: String| {
                oninput.call(if hidden { FieldText::Secret(Hidden::new(next)) } else { FieldText::Plain(next) })
            },
        }
    }
}

/// A pop-up button over a field's choices, the chosen slug the host's. A pick reports the slug.
#[component]
pub(crate) fn Pick(
    label: String,
    choices: Vec<Choice>,
    chosen: Option<String>,
    onpick: EventHandler<String>,
) -> Element {
    let items = choices
        .into_iter()
        .map(|choice| MenuItem::new(choice.slug, choice.label))
        .collect();
    rsx! {
        PopUpButton { items, value: chosen, title: Some(label), onpick }
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
    /// Its letter or its favicon.
    pub(crate) style: MarkStyle,
    /// Its letter and colour as data, drawn instead of the provider's own letter.
    pub(crate) face: Option<MarkFace>,
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
                        leading: mark_leading(row.mark, &row.style, row.face.as_ref()),
                        size: RowSize::Settings,
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
    let answers = match intent {
        Intent::Default => Answers::Return,
        Intent::Cancel => Answers::Escape,
        Intent::Plain => Answers::Nothing,
    };
    let focus = match landing {
        Landing::Here => ButtonFocus::OnMount,
        Landing::Anywhere => ButtonFocus::Manual,
    };
    rsx! {
        Button {
            label,
            answers,
            role: ButtonRole::Normal,
            focus,
            availability,
            onclick: move |_| onclick.call(()),
        }
    }
}

/// A button that copies `text`: the host does the copying (it hears `oncopy`; ds-shell may not
/// depend on ds-blitz, so `ds_blitz::clipboard::write_text` is the host's call), and `state` is
/// the host's word on whether it has, so the label can say "Copied".
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
        Action {
            label: shown,
            intent,
            landing,
            onclick: move |()| oncopy.call(text.clone()),
        }
    }
}

/// A pop-up button over accounts and "Add Account...", its value the host's. Its open state is its
/// own: no host or sheet needs to hold it.
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

/// The favicon size a settings row draws: `--row-avatar` (row.css), so the image is supplied at
/// the size it is painted at.
const ROW_AVATAR_PX: u8 = 32;

/// The favicon `style` holds, drawn `px` across, when it is one a document can load and the
/// provider has a favicon to show (a local account never does).
fn favicon(provider: MarkProvider, style: &MarkStyle, px: u8) -> Option<IconSource> {
    match style {
        MarkStyle::Image(src) if provider != MarkProvider::Local => {
            IconUrl::parse(&src.0).ok().map(|url| {
                IconSource::Image(ExternalIcon {
                    url,
                    size: IconSize::Px(IconPx(px)),
                })
            })
        }
        MarkStyle::Image(_) | MarkStyle::Letter => None,
    }
}

/// What leads a row of providers: the favicon when `style` holds one, else the data `face`'s
/// disc, else the provider's own letter's.
pub(crate) fn mark_leading(
    provider: MarkProvider,
    style: &MarkStyle,
    face: Option<&MarkFace>,
) -> RowLeading {
    let lettered = || {
        face.and_then(|face| face.avatar(AvatarSize::Size28))
            .unwrap_or_else(|| provider.avatar(AvatarSize::Size28))
    };
    match favicon(provider, style, ROW_AVATAR_PX) {
        Some(source) => RowLeading::Source(source),
        None => RowLeading::Avatar(lettered()),
    }
}

/// A provider's round mark, `size` across: the leading mark of a header or a line. Under
/// `MarkStyle::Image` it is the provider's favicon.
#[component]
pub(crate) fn Disc(
    provider: MarkProvider,
    size: AvatarSize,
    #[props(default)] style: MarkStyle,
) -> Element {
    let px = match size {
        AvatarSize::Size28 => 28,
        _ => 48,
    };
    match favicon(provider, &style, px) {
        Some(source) => rsx! {
            span { class: "ds-acc-disc", "data-kind": "image",
                IconView { source }
            }
        },
        None => ds::components::content::avatar::face(provider.avatar(size)),
    }
}
