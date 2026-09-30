//! EmptyState: what a list, a pane or a page says when there is nothing to show (design/30
//! section 2.9, SwiftUI `ContentUnavailableView`): a glyph, a title, an optional description and
//! an optional action, centred.
//!
//! Three forms (`EmptyForm`): `Empty` (nothing here yet), `NoResults` (a search or filter found
//! nothing) and `Failure` (it could not be loaded, with a Retry when the caller can). It is static
//! text: nothing moves, and a failed secure entry's shake is the entry's, not this component's.

use crate::components::content::icon_view::IconView;
use crate::components::controls::button::Button;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::word::Word;
use ds_style::icon::Icon;
use ds_style::icon::render::IconSize;

/// Why there is nothing to show: `data-form`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum EmptyForm {
    /// Nothing here yet.
    #[default]
    Empty,
    /// A search or a filter found nothing.
    NoResults,
    /// Loading failed.
    Failure,
}

/// The glyph a form draws when the caller brings none.
fn glyph_of(form: EmptyForm) -> Icon {
    match form {
        EmptyForm::Empty => Icon::Inbox,
        EmptyForm::NoResults => Icon::Search,
        EmptyForm::Failure => Icon::Refresh,
    }
}

/// A placeholder for nothing to show. `icon` overrides the form's glyph; `description` is the
/// line under the title; `action` is an element of the caller's (a button that adds the first
/// item); a `Failure` with `onretry` draws a Retry button, after the action. `common` puts the
/// consumer's `id`, `data-*` and classes on the root, and `aria_label` names it in place of its
/// title.
#[component]
pub fn EmptyState(
    #[props(default)] form: EmptyForm,
    #[props(into)] title: String,
    #[props(default)] description: Option<String>,
    #[props(default)] icon: Option<Icon>,
    #[props(default)] action: Option<Element>,
    #[props(default)] onretry: Option<EventHandler<()>>,
    #[props(default)] common: Common,
) -> Element {
    let class = common.class("ds-empty-state");
    let data = common.data_attributes();
    let icon = icon.unwrap_or_else(|| glyph_of(form));
    let retry = onretry.filter(|_| form == EmptyForm::Failure);
    rsx! {
        div {
            id: common.id.clone(),
            class,
            role: "status",
            "aria-label": common.aria_label.clone(),
            "data-form": form.slug(),
            onmounted: move |event| common.mounted(event),
            ..data,
            div { class: "ds-empty-state-icon",
                IconView { source: icon.into(), size: IconSize::Tile48 }
            }
            div { class: "ds-empty-state-title", "{title}" }
            if let Some(description) = description {
                div { class: "ds-empty-state-body", "{description}" }
            }
            if action.is_some() || retry.is_some() {
                div { class: "ds-empty-state-action",
                    if let Some(action) = action {
                        {action}
                    }
                    if let Some(retry) = retry {
                        Button {
                            label: "Retry".to_owned(),
                            onclick: move |_| retry.call(()),
                        }
                    }
                }
            }
        }
    }
}
