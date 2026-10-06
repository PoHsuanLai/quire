//! FormSection: a titled group of rows on the `--grp` ground, with a help footer under it
//! (design/34-MODERN-LOOK.md sections 2.3 and 3.5). The group is one rounded card with no outer
//! line; the rows inside it (`FieldRow`s, or a grouped `List`) are separated by a hairline that
//! starts at the row's text and none above the first or below the last.
//!
//! Markup: `section.ds-form-section` of an optional `div.ds-form-section-title`,
//! `div.ds-form-section-group` and an optional `p.ds-form-section-footer`.

use crate::root::common::Common;
use dioxus::prelude::*;

/// A section: `title` over the group (13 px, 600, secondary), the rows in `children`, `footer`
/// under it in the help type.
#[component]
pub fn FormSection(
    #[props(default)] title: Option<String>,
    #[props(default)] footer: Option<String>,
    children: Element,
    #[props(default)] common: Common,
) -> Element {
    let data = common.data_attributes();
    rsx! {
        section {
            id: common.id.clone(),
            class: common.class("ds-form-section"),
            "aria-label": common.aria_label.clone().or_else(|| title.clone()),
            onmounted: move |event| common.mounted(event),
            ..data,
            if let Some(title) = &title {
                div { class: "ds-form-section-title", "{title}" }
            }
            div { class: "ds-form-section-group", {children} }
            if let Some(footer) = &footer {
                p { class: "ds-form-section-footer", "{footer}" }
            }
        }
    }
}
