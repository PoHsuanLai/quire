//! FieldRow and FieldGroup: a form's rows (`NSGridView`, design/30 section 2.2). A row is a
//! label column and a control column with a help line under the label; a group is the inset card
//! rows sit in, a hairline between them, with an optional title above.
//!
//! `RowLayout::Setting` is System Settings' row: the label at the start growing, the control at
//! the end, 36 px tall. `RowLayout::Form` is a grid's: a fixed label column ending at the
//! control, so a stack of rows lines its controls up.
//!
//! Markup: `div.ds-field-row[data-layout]` of `span.ds-field-row-label` (`.ds-field-row-title`,
//! `.ds-field-row-help`) and `div.ds-field-row-control`; `section.ds-field-group` of an optional
//! `SectionHeader` and `div.ds-field-group-rows`.

use crate::components::content::text_runs::{TextLine, text};
use crate::components::lists::section_header::SectionHeader;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::Availability;
use ds_core::word::Word;

/// How a row lays its label and control out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum RowLayout {
    /// The label grows from the start, the control sits at the end.
    #[default]
    Setting,
    /// The label in a fixed column, the control beside it.
    Form,
}

/// One row of a form: `label`, the `help` line under it, and the controls in `children`: one, or
/// several that wrap onto further lines when the row is too narrow for them. A
/// disabled or busy row dims its label and takes no pointer.
#[component]
pub fn FieldRow(
    #[props(into)] label: TextLine,
    #[props(default)] help: Option<TextLine>,
    #[props(default)] layout: RowLayout,
    #[props(default)] availability: Availability,
    children: Element,
    #[props(default)] common: Common,
) -> Element {
    let data = common.data_attributes();
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-field-row"),
            role: "group",
            "aria-label": common.aria_label.clone().unwrap_or_else(|| label.plain_text()),
            "aria-disabled": availability.aria_disabled(),
            "aria-busy": availability.aria_busy(),
            "data-layout": layout.slug(),
            "data-availability": availability.slug(),
            onmounted: move |event| common.mounted(event),
            ..data,
            span { class: "ds-field-row-label",
                span { class: "ds-field-row-title", {text(&label)} }
                if let Some(help) = help.as_ref() {
                    span { class: "ds-field-row-help", {text(help)} }
                }
            }
            div { class: "ds-field-row-control", {children} }
        }
    }
}

/// The card a form's rows sit in, with `title` above it when there is one.
#[component]
pub fn FieldGroup(
    #[props(default)] title: Option<String>,
    children: Element,
    #[props(default)] common: Common,
) -> Element {
    let data = common.data_attributes();
    rsx! {
        section {
            id: common.id.clone(),
            class: common.class("ds-field-group"),
            "aria-label": common.aria_label.clone().or_else(|| title.clone()),
            onmounted: move |event| common.mounted(event),
            ..data,
            if let Some(title) = title.clone() {
                SectionHeader { title }
            }
            div { class: "ds-field-group-rows", {children} }
        }
    }
}
